<script lang="ts">
  import { isTypingTarget } from '$lib/shortcuts';
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    listRemoteDir,
    readRemoteFile,
    writeRemoteFile,
    mkdirRemoteDir,
    deleteRemoteFile,
    renameRemoteFile,
    copyRemoteFile,
    sftpChmod,
    sftpUpload,
    sftpDownload,
    sftpCancel,
    sftpCompress,
    sftpExtract,
    searchRemoteFiles,
    type RemoteSearchItem,
    type SftpFileEntry,
    type SftpProgressPayload
  } from '$lib/api/sftp';
  import {
    localListDir,
    localStat,
    localMkdir,
    localDelete,
    localRename,
    localReadFile,
    localWriteFile,
    type LocalFileEntry
  } from '$lib/api/local_fs';
  import { listHosts, saveHost, type HostRecord, type ConnectionProtocol } from '$lib/api/hosts';
  import type RemoteFileEditorComponent from '$lib/components/RemoteFileEditor.svelte';
  import DirectorySync from '$lib/components/DirectorySync.svelte';
  import ProGate from '$lib/components/ProGate.svelte';
  import PageContainer from '$lib/components/PageContainer.svelte';
  import { PageHeader, Button, Input, Badge, Modal } from '@cecepazhar/caui';
  import { t, intlLocale } from '$lib/i18n/index.svelte';

  interface TransferItem {
    id: string;
    source: string;
    target: string;
    direction: 'upload' | 'download';
    status: 'queued' | 'active' | 'completed' | 'failed' | 'cancelled';
    bytesTransferred: number;
    totalBytes: number;
    speedBps: number;
    error?: string;
  }

  // Hosts & Navigation
  let hosts = $state<HostRecord[]>([]);
  let currentHostId = $state('');
  let viewMode = $state<'dual' | 'single'>('dual');
  let activePane = $state<'local' | 'remote'>('remote');

  // Protocol & Quick Connect State
  let showProtocolSelector = $state(false);
  let qcAddress = $state('');
  let qcPort = $state<number>(22);
  let qcUsername = $state('root');
  let qcPassword = $state('');
  let qcProtocol = $state<ConnectionProtocol>('ssh');
  let qcLabel = $state('');
  let qcConnecting = $state(false);
  let qcError = $state('');

  let currentHost = $derived(hosts.find((h) => h.id === currentHostId));
  let activeProtocolBadge = $derived.by(() => {
    const proto = currentHost?.protocol;
    if (proto === 's3') return 'S3';
    if (proto === 'webdav') return 'WebDAV';
    if (proto === 'ftp' || proto === 'ftps') return 'FTP';
    return 'SFTP';
  });

  async function switchCurrentProtocol(proto: ConnectionProtocol) {
    if (!currentHost) return;
    try {
      await saveHost({
        id: currentHost.id,
        label: currentHost.label,
        address: currentHost.address,
        port: currentHost.port,
        username: currentHost.username,
        authMethod: currentHost.authMethod,
        tags: currentHost.tags,
        os: currentHost.os,
        protocol: proto,
      });
      currentHost.protocol = proto;
      showProtocolSelector = false;
      await fetchRemoteFiles();
    } catch (e: any) {
      errorMsg = e?.message || String(e);
    }
  }

  async function handleQuickConnect() {
    if (!qcAddress.trim()) {
      qcError = t('sftp.errAddress');
      return;
    }
    qcConnecting = true;
    qcError = '';
    try {
      const label = qcLabel.trim() || `${qcProtocol.toUpperCase()} - ${qcAddress.trim()}`;
      const defaultPort = qcProtocol === 'ftp' ? 21 : (qcProtocol === 's3' || qcProtocol === 'webdav' ? 443 : 22);
      const newHost = await saveHost({
        label,
        address: qcAddress.trim(),
        port: Number(qcPort) || defaultPort,
        username: qcUsername.trim() || 'root',
        authMethod: { type: 'password' },
        tags: ['quick-connect'],
        protocol: qcProtocol,
        secret: qcPassword ? qcPassword : undefined,
      });
      hosts = await listHosts();
      currentHostId = newHost.id;
      showProtocolSelector = false;
      await fetchRemoteFiles();
    } catch (e: any) {
      qcError = e?.message || String(e);
    } finally {
      qcConnecting = false;
    }
  }

  // Local Pane State
  let localPath = $state('~');
  let localFiles = $state<LocalFileEntry[]>([]);
  let localLoading = $state(false);
  let localSelectedPaths = $state<Set<string>>(new Set());
  let localLastSelected = $state<LocalFileEntry | null>(null);

  // Remote Pane State
  let remotePath = $state('/');
  let remoteFiles = $state<SftpFileEntry[]>([]);
  let remoteLoading = $state(false);
  let remoteSelectedPaths = $state<Set<string>>(new Set());
  let remoteLastSelected = $state<SftpFileEntry | null>(null);

  // Notifications
  let errorMsg = $state('');
  let successMsg = $state('');

  // Transfer Queue State
  let transfers = $state<TransferItem[]>([]);
  let isQueueRunning = $state(false);
  let unlistenProgress: UnlistenFn | null = null;

  // Modals
  let showNewFolderModal = $state(false);
  let newFolderName = $state('');
  let newFolderTargetPane = $state<'local' | 'remote'>('remote');

  let showRenameModal = $state(false);
  let renameItem = $state<{ pane: 'local' | 'remote'; path: string; name: string } | null>(null);
  let renameNewName = $state('');

  let showDeleteModal = $state(false);
  let deleteTarget = $state<{ pane: 'local' | 'remote'; paths: string[] } | null>(null);

  let showEditorModal = $state(false);
  let editorItem = $state<{ pane: 'local' | 'remote'; path: string; name: string } | null>(null);
  let editorContent = $state('');
  let isEditorSaving = $state(false);
  let isEditorLoading = $state(false);

  let showChmodModal = $state(false);
  let chmodTarget = $state<SftpFileEntry | null>(null);
  let chmodOctal = $state('0755');
  let chmodUserR = $state(true);
  let chmodUserW = $state(true);
  let chmodUserX = $state(true);
  let chmodGroupR = $state(true);
  let chmodGroupW = $state(false);
  let chmodGroupX = $state(true);
  let chmodOtherR = $state(true);
  let chmodOtherW = $state(false);
  let chmodOtherX = $state(true);

  // Archive modals
  let showCompressModal = $state(false);
  let compressItems = $state<string[]>([]);
  let compressParentDir = $state('');
  let compressArchiveName = $state('archive.tar.gz');
  let isCompressing = $state(false);

  let showExtractModal = $state(false);
  let extractArchivePath = $state('');
  let extractDestDir = $state('');
  let isExtracting = $state(false);

  // Directory Sync modal
  let showSyncModal = $state(false);

  // Remote Search State
  let remoteSearchQuery = $state("");
  let isSearchingRemote = $state(false);
  let remoteSearchResults = $state<RemoteSearchItem[]>([]);
  let showRemoteSearch = $state(false);
  let searchMinSizeKB = $state<string>("");
  let searchMaxSizeKB = $state<string>("");

  async function executeRemoteSearch() {
    if (!currentHostId) return;
    const q = remoteSearchQuery.trim();
    if (!q) return;
    isSearchingRemote = true;
    try {
      const minBytes = searchMinSizeKB ? Number(searchMinSizeKB) * 1024 : undefined;
      const maxBytes = searchMaxSizeKB ? Number(searchMaxSizeKB) * 1024 : undefined;
      remoteSearchResults = await searchRemoteFiles(
        currentHostId,
        remotePath,
        q.includes("*") || q.includes("?") ? q : `*${q}*`,
        100,
        minBytes,
        maxBytes
      );
    } catch (e: unknown) {
      errorMsg = String(e);
    } finally {
      isSearchingRemote = false;
    }
  }

  function jumpToSearchResult(item: RemoteSearchItem) {
    if (item.is_dir) {
      showRemoteSearch = false;
      navigateRemote(item.path);
    } else {
      const parent = item.path.substring(0, item.path.lastIndexOf("/")) || "/";
      showRemoteSearch = false;
      remotePath = parent;
      fetchRemoteFiles().then(() => {
        remoteLastSelected = remoteFiles.find((f) => f.path === item.path) ?? null;
        if (remoteLastSelected) {
          remoteSelectedPaths = new Set([remoteLastSelected.path]);
        }
      });
    }
  }


  // Context menu — shared for both local & remote panes
  type CtxEntry = (SftpFileEntry | LocalFileEntry) & { pane?: 'local' | 'remote' };
  let ctxMenu = $state<{ x: number; y: number; item: CtxEntry; pane: 'local' | 'remote' } | null>(null);

  function openRemoteCtxMenu(e: MouseEvent, item: SftpFileEntry) {
    e.preventDefault();
    e.stopPropagation();
    activePane = 'remote';
    if (!remoteSelectedPaths.has(item.path)) {
      remoteSelectedPaths.clear();
      remoteSelectedPaths.add(item.path);
      remoteLastSelected = item;
    }
    ctxMenu = { x: e.clientX, y: e.clientY, item: item as CtxEntry, pane: 'remote' };
  }

  function openLocalCtxMenu(e: MouseEvent, item: LocalFileEntry) {
    e.preventDefault();
    e.stopPropagation();
    activePane = 'local';
    if (!localSelectedPaths.has(item.path)) {
      localSelectedPaths.clear();
      localSelectedPaths.add(item.path);
      localLastSelected = item as LocalFileEntry;
    }
    ctxMenu = { x: e.clientX, y: e.clientY, item: item as CtxEntry, pane: 'local' };
  }

  /** @deprecated use openRemoteCtxMenu / openLocalCtxMenu */
  function openCtxMenu(e: MouseEvent, item: SftpFileEntry) {
    openRemoteCtxMenu(e, item);
  }

  function openCompressModal() {
    const selected = [...remoteSelectedPaths];
    if (selected.length === 0 && remoteLastSelected) selected.push(remoteLastSelected.path);
    if (selected.length === 0) { errorMsg = t('sftp.errSelectCompress'); return; }
    compressItems = selected.map((p) => p.split('/').pop() ?? p);
    compressParentDir = remotePath;
    compressArchiveName = 'archive.tar.gz';
    showCompressModal = true;
  }

  async function confirmCompress() {
    if (!currentHostId || compressItems.length === 0) return;
    isCompressing = true;
    try {
      await sftpCompress(currentHostId, compressParentDir, compressItems, compressArchiveName);
      showCompressModal = false;
      notifySuccess(t('sftp.compressed', { name: compressArchiveName }));
      await fetchRemoteFiles();
    } catch (e: unknown) {
      errorMsg = String(e);
    } finally {
      isCompressing = false;
    }
  }

  function openExtractHere(item: SftpFileEntry) {
    extractArchivePath = item.path;
    extractDestDir = remotePath;
    showExtractModal = true;
  }

  function openExtractTo(item: SftpFileEntry) {
    extractArchivePath = item.path;
    const base = item.name.replace(/\.(tar\.gz|tar\.bz2|tar\.xz|tgz|tar|zip)$/i, '');
    extractDestDir = `${remotePath}/${base}`;
    showExtractModal = true;
  }

  async function confirmExtract() {
    if (!currentHostId) return;
    isExtracting = true;
    try {
      await sftpExtract(currentHostId, extractArchivePath, extractDestDir);
      showExtractModal = false;
      notifySuccess(t('sftp.extracted', { dest: extractDestDir }));
      await fetchRemoteFiles();
    } catch (e: unknown) {
      errorMsg = String(e);
    } finally {
      isExtracting = false;
    }
  }

  function isArchive(name: string): boolean {
    return /\.(tar\.gz|tgz|tar\.bz2|tar\.xz|tar|zip)$/i.test(name);
  }

  function closeCtxMenu() { ctxMenu = null; }

  function notifySuccess(msg: string) {
    successMsg = msg;
    setTimeout(() => {
      if (successMsg === msg) successMsg = '';
    }, 4000);
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatSpeed(bps: number): string {
    return `${formatSize(bps)}/s`;
  }

  function formatMtime(mtime: number): string {
    if (!mtime) return '-';
    return new Date(mtime * 1000).toLocaleString(intlLocale(), {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatPermissions(mode?: number): string {
    if (mode === undefined || mode === 0) return '---------';
    const octal = (mode & 0o777).toString(8).padStart(3, '0');
    const rwx = (d: number) => {
      let s = '';
      s += d & 4 ? 'r' : '-';
      s += d & 2 ? 'w' : '-';
      s += d & 1 ? 'x' : '-';
      return s;
    };
    return (
      rwx(parseInt(octal[0] || '0', 8)) +
      rwx(parseInt(octal[1] || '0', 8)) +
      rwx(parseInt(octal[2] || '0', 8))
    );
  }

  function joinPath(dir: string, name: string): string {
    if (dir === '/' || dir === '.' || !dir) return `/${name}`;
    return `${dir.replace(/\/+$/, '')}/${name}`;
  }

  // --- LOCAL FS LOGIC ---
  async function fetchLocalFiles() {
    localLoading = true;
    errorMsg = '';
    try {
      localFiles = await localListDir(localPath);
      // Canonicalize display path if needed
      if (localFiles.length > 0 && localPath === '~') {
        const parent = localFiles[0].path.substring(0, localFiles[0].path.lastIndexOf('/')) || '~';
        localPath = parent;
      }
    } catch (e: any) {
      errorMsg = t('sftp.errLocalFs', { error: e?.message || String(e) });
      localFiles = [];
    } finally {
      localLoading = false;
    }
  }

  function navigateLocal(path: string) {
    localPath = path;
    localSelectedPaths = new Set();
    fetchLocalFiles();
  }

  function goUpLocal() {
    if (localPath === '/' || localPath === '') return;
    const parts = localPath.split('/').filter(Boolean);
    parts.pop();
    localPath = '/' + parts.join('/');
    if (localPath === '') localPath = '/';
    fetchLocalFiles();
  }

  /** Splits an absolute or `~`-prefixed path into clickable breadcrumb segments. */
  function localBreadcrumbs(): string[] {
    if (!localPath || localPath === '~') return [];
    return localPath.split('/').filter(Boolean);
  }

  function navigateToBreadcrumbLocal(index: number) {
    const parts = localBreadcrumbs();
    const target = parts.slice(0, index + 1).join('/');
    navigateLocal('/' + target);
  }

  function navigateToBreadcrumbRemote(index: number) {
    const parts = remotePath.split('/').filter(Boolean);
    const target = parts.slice(0, index + 1).join('/');
    navigateRemote('/' + target);
  }

  function remoteBreadcrumbs(): string[] {
    if (!remotePath || remotePath === '/') return [];
    return remotePath.split('/').filter(Boolean);
  }

  // --- REMOTE SFTP LOGIC ---
  async function fetchRemoteFiles() {
    if (!currentHostId) return;
    remoteLoading = true;
    errorMsg = '';
    try {
      remoteFiles = await listRemoteDir(currentHostId, remotePath);
    } catch (e: any) {
      errorMsg = t('sftp.errRemote', { error: e?.message || String(e) });
      remoteFiles = [];
    } finally {
      remoteLoading = false;
    }
  }

  function navigateRemote(path: string) {
    remotePath = path;
    remoteSelectedPaths = new Set();
    fetchRemoteFiles();
  }

  function goUpRemote() {
    if (remotePath === '/' || remotePath === '') return;
    const parts = remotePath.split('/').filter(Boolean);
    parts.pop();
    remotePath = '/' + parts.join('/');
    if (remotePath === '') remotePath = '/';
    fetchRemoteFiles();
  }

  async function loadHosts() {
    try {
      hosts = await listHosts();
      const hostParam = page.url.searchParams.get('host');
      if (hostParam && hosts.some((h) => h.id === hostParam)) {
        currentHostId = hostParam;
      } else if (hosts.length > 0 && !currentHostId) {
        currentHostId = hosts[0].id;
      }
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  // --- SELECTION HELPERS ---
  function toggleLocalSelect(file: LocalFileEntry) {
    const next = new Set(localSelectedPaths);
    if (next.has(file.path)) {
      next.delete(file.path);
    } else {
      next.add(file.path);
    }
    localSelectedPaths = next;
    localLastSelected = file;
    activePane = 'local';
  }

  function toggleRemoteSelect(file: SftpFileEntry) {
    const next = new Set(remoteSelectedPaths);
    if (next.has(file.path)) {
      next.delete(file.path);
    } else {
      next.add(file.path);
    }
    remoteSelectedPaths = next;
    remoteLastSelected = file;
    activePane = 'remote';
  }

  // --- QUEUE & TRANSFER ENGINE ---
  function enqueueTransfer(
    direction: 'upload' | 'download',
    source: string,
    target: string
  ) {
    const id = `tx_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
    transfers = [
      ...transfers,
      {
        id,
        direction,
        source,
        target,
        status: 'queued',
        bytesTransferred: 0,
        totalBytes: 0,
        speedBps: 0
      }
    ];
    runTransferQueue();
  }

  async function runTransferQueue() {
    if (isQueueRunning) return;
    isQueueRunning = true;

    while (true) {
      const nextIndex = transfers.findIndex((t) => t.status === 'queued');
      if (nextIndex === -1) break;

      const item = transfers[nextIndex];
      transfers[nextIndex].status = 'active';

      try {
        if (item.direction === 'upload') {
          await sftpUpload(currentHostId, item.source, item.target, item.id);
        } else {
          await sftpDownload(currentHostId, item.source, item.target, item.id);
        }
        transfers[nextIndex].status = 'completed';
        transfers[nextIndex].bytesTransferred = transfers[nextIndex].totalBytes;
      } catch (err: any) {
        const msg = String(err?.message || err);
        if (msg.includes('cancelled')) {
          transfers[nextIndex].status = 'cancelled';
        } else {
          transfers[nextIndex].status = 'failed';
          transfers[nextIndex].error = msg;
        }
      }
    }

    isQueueRunning = false;
    // Refresh both panes
    await Promise.all([fetchLocalFiles(), fetchRemoteFiles()]);
  }

  async function cancelQueueItem(id: string) {
    try {
      await sftpCancel(id);
      const idx = transfers.findIndex((t) => t.id === id);
      if (idx !== -1 && transfers[idx].status === 'active') {
        transfers[idx].status = 'cancelled';
      }
    } catch (e: any) {
      errorMsg = t('sftp.errCancel', { error: e?.message || String(e) });
    }
  }

  function clearCompletedTransfers() {
    transfers = transfers.filter((t) => t.status === 'active' || t.status === 'queued');
  }

  // --- ACTIONS (F5: Transfer, F6: Move, F7: Mkdir, F8: Delete, F2: Rename, F4: Edit) ---
  function handleCopyTransfer() {
    if (activePane === 'local') {
      // Upload selected local files to remote current dir
      if (localSelectedPaths.size === 0 && localLastSelected) {
        localSelectedPaths.add(localLastSelected.path);
      }
      for (const p of localSelectedPaths) {
        const name = p.split('/').pop() || 'file';
        const remoteTarget = joinPath(remotePath, name);
        enqueueTransfer('upload', p, remoteTarget);
      }
      notifySuccess(t('sftp.queuedUpload', { count: localSelectedPaths.size }));
      localSelectedPaths = new Set();
    } else {
      // Download selected remote files to local current dir
      if (remoteSelectedPaths.size === 0 && remoteLastSelected) {
        remoteSelectedPaths.add(remoteLastSelected.path);
      }
      for (const p of remoteSelectedPaths) {
        const name = p.split('/').pop() || 'file';
        const localTarget = joinPath(localPath, name);
        enqueueTransfer('download', p, localTarget);
      }
      notifySuccess(t('sftp.queuedDownload', { count: remoteSelectedPaths.size }));
      remoteSelectedPaths = new Set();
    }
  }

  function openNewFolderModal(pane: 'local' | 'remote') {
    newFolderTargetPane = pane;
    newFolderName = '';
    showNewFolderModal = true;
  }

  async function confirmNewFolder() {
    if (!newFolderName.trim()) return;
    const folder = newFolderName.trim();
    showNewFolderModal = false;

    try {
      if (newFolderTargetPane === 'local') {
        const dest = joinPath(localPath, folder);
        await localMkdir(dest);
        notifySuccess(t('sftp.createdLocalFolder', { name: folder }));
        await fetchLocalFiles();
      } else {
        const dest = joinPath(remotePath, folder);
        await mkdirRemoteDir(currentHostId, dest);
        notifySuccess(t('sftp.createdRemoteFolder', { name: folder }));
        await fetchRemoteFiles();
      }
    } catch (e: any) {
      errorMsg = t('sftp.errCreateFolder', { error: e?.message || String(e) });
    }
  }

  let showNewFileModal = $state(false);
  let newFileTargetPane = $state<'local' | 'remote'>('remote');
  let newFileName = $state('');

  function openNewFileModal(pane: 'local' | 'remote') {
    newFileTargetPane = pane;
    newFileName = '';
    showNewFileModal = true;
  }

  async function confirmNewFile() {
    if (!newFileName.trim()) return;
    const filename = newFileName.trim();
    showNewFileModal = false;

    try {
      if (newFileTargetPane === 'local') {
        const dest = joinPath(localPath, filename);
        await localWriteFile(dest, []);
        notifySuccess(t('sftp.createdLocalFile', { name: filename }));
        await fetchLocalFiles();
      } else {
        const dest = joinPath(remotePath, filename);
        await writeRemoteFile(currentHostId, dest, []);
        notifySuccess(t('sftp.createdRemoteFile', { name: filename }));
        await fetchRemoteFiles();
      }
    } catch (e: any) {
      errorMsg = t('sftp.errCreateFile', { error: e?.message || String(e) });
    }
  }

  function openRenameModal() {
    if (activePane === 'local' && localLastSelected) {
      renameItem = { pane: 'local', path: localLastSelected.path, name: localLastSelected.name };
      renameNewName = localLastSelected.name;
      showRenameModal = true;
    } else if (activePane === 'remote' && remoteLastSelected) {
      renameItem = { pane: 'remote', path: remoteLastSelected.path, name: remoteLastSelected.name };
      renameNewName = remoteLastSelected.name;
      showRenameModal = true;
    }
  }

  async function confirmRename() {
    if (!renameItem || !renameNewName.trim() || renameNewName.trim() === renameItem.name) {
      showRenameModal = false;
      return;
    }
    const newName = renameNewName.trim();
    const parent = renameItem.path.substring(0, renameItem.path.lastIndexOf('/')) || '/';
    const newPath = joinPath(parent, newName);
    showRenameModal = false;

    try {
      if (renameItem.pane === 'local') {
        await localRename(renameItem.path, newPath);
        notifySuccess(t('sftp.renamedLocal', { name: newName }));
        await fetchLocalFiles();
      } else {
        await renameRemoteFile(currentHostId, renameItem.path, newPath);
        notifySuccess(t('sftp.renamedRemote', { name: newName }));
        await fetchRemoteFiles();
      }
    } catch (e: any) {
      errorMsg = t('sftp.errRename', { error: e?.message || String(e) });
    }
  }

  function openDeleteModal() {
    if (activePane === 'local' && localSelectedPaths.size > 0) {
      deleteTarget = { pane: 'local', paths: Array.from(localSelectedPaths) };
      showDeleteModal = true;
    } else if (activePane === 'local' && localLastSelected) {
      deleteTarget = { pane: 'local', paths: [localLastSelected.path] };
      showDeleteModal = true;
    } else if (activePane === 'remote' && remoteSelectedPaths.size > 0) {
      deleteTarget = { pane: 'remote', paths: Array.from(remoteSelectedPaths) };
      showDeleteModal = true;
    } else if (activePane === 'remote' && remoteLastSelected) {
      deleteTarget = { pane: 'remote', paths: [remoteLastSelected.path] };
      showDeleteModal = true;
    }
  }

  async function confirmDelete() {
    if (!deleteTarget || deleteTarget.paths.length === 0) {
      showDeleteModal = false;
      return;
    }
    showDeleteModal = false;

    try {
      if (deleteTarget.pane === 'local') {
        for (const p of deleteTarget.paths) {
          await localDelete(p, true, true);
        }
        notifySuccess(t('sftp.deletedLocal', { count: deleteTarget.paths.length }));
        localSelectedPaths = new Set();
        localLastSelected = null;
        await fetchLocalFiles();
      } else {
        for (const p of deleteTarget.paths) {
          await deleteRemoteFile(currentHostId, p, true, true);
        }
        notifySuccess(t('sftp.deletedRemote', { count: deleteTarget.paths.length }));
        remoteSelectedPaths = new Set();
        remoteLastSelected = null;
        await fetchRemoteFiles();
      }
    } catch (e: any) {
      errorMsg = t('sftp.errDelete', { error: e?.message || String(e) });
    }
  }

  // CodeMirror (~650 KB with its language packs) loads the first time a file is opened,
  // not with the page — most visits to Files never open the editor.
  let RemoteFileEditor = $state<typeof RemoteFileEditorComponent | null>(null);

  async function openEditorModal() {
    const isLocal = activePane === 'local';
    const target = isLocal ? localLastSelected : remoteLastSelected;
    if (!target || target.is_dir) return;
    if (!RemoteFileEditor) {
      import('$lib/components/RemoteFileEditor.svelte').then((mod) => (RemoteFileEditor = mod.default));
    }

    editorItem = { pane: isLocal ? 'local' : 'remote', path: target.path, name: target.name };
    showEditorModal = true;
    isEditorLoading = true;
    editorContent = '';

    try {
      let bytes: number[] = [];
      if (isLocal) {
        bytes = await localReadFile(target.path);
      } else {
        bytes = await readRemoteFile(currentHostId, target.path);
      }
      editorContent = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
    } catch (e: any) {
      errorMsg = t('sftp.errRead', { error: e?.message || String(e) });
      showEditorModal = false;
    } finally {
      isEditorLoading = false;
    }
  }

  async function saveEditorFile() {
    if (!editorItem) return;
    isEditorSaving = true;
    try {
      const bytes = Array.from(new TextEncoder().encode(editorContent));
      if (editorItem.pane === 'local') {
        await localWriteFile(editorItem.path, bytes);
        notifySuccess(t('sftp.savedLocal', { name: editorItem.name }));
        await fetchLocalFiles();
      } else {
        await writeRemoteFile(currentHostId, editorItem.path, bytes);
        notifySuccess(t('sftp.savedRemote', { name: editorItem.name }));
        await fetchRemoteFiles();
      }
      showEditorModal = false;
    } catch (e: any) {
      errorMsg = t('sftp.errSave', { error: e?.message || String(e) });
    } finally {
      isEditorSaving = false;
    }
  }

  // --- CHMOD MODAL ---
  function openChmodModal(file: SftpFileEntry) {
    chmodTarget = file;
    const mode = file.mode || 0o755;
    const oct = (mode & 0o777).toString(8).padStart(3, '0');
    chmodOctal = oct;
    updateChmodCheckboxesFromOctal(oct);
    showChmodModal = true;
  }

  function updateChmodCheckboxesFromOctal(oct: string) {
    const u = parseInt(oct[0] || '0', 8);
    const g = parseInt(oct[1] || '0', 8);
    const o = parseInt(oct[2] || '0', 8);
    chmodUserR = !!(u & 4);
    chmodUserW = !!(u & 2);
    chmodUserX = !!(u & 1);
    chmodGroupR = !!(g & 4);
    chmodGroupW = !!(g & 2);
    chmodGroupX = !!(g & 1);
    chmodOtherR = !!(o & 4);
    chmodOtherW = !!(o & 2);
    chmodOtherX = !!(o & 1);
  }

  function updateChmodOctalFromCheckboxes() {
    const u = (chmodUserR ? 4 : 0) + (chmodUserW ? 2 : 0) + (chmodUserX ? 1 : 0);
    const g = (chmodGroupR ? 4 : 0) + (chmodGroupW ? 2 : 0) + (chmodGroupX ? 1 : 0);
    const o = (chmodOtherR ? 4 : 0) + (chmodOtherW ? 2 : 0) + (chmodOtherX ? 1 : 0);
    chmodOctal = `${u}${g}${o}`;
  }

  async function confirmChmod() {
    if (!chmodTarget) return;
    showChmodModal = false;
    try {
      const mode = parseInt(chmodOctal, 8);
      await sftpChmod(currentHostId, chmodTarget.path, mode);
      notifySuccess(t('sftp.chmodDone', { name: chmodTarget.name, mode: chmodOctal }));
      await fetchRemoteFiles();
    } catch (e: any) {
      errorMsg = t('sftp.errChmod', { error: e?.message || String(e) });
    }
  }

  // --- DRAG & DROP ---
  function onDragStart(event: DragEvent, sourcePane: 'local' | 'remote', item: LocalFileEntry | SftpFileEntry) {
    if (!event.dataTransfer) return;
    event.dataTransfer.setData('text/plain', JSON.stringify({ pane: sourcePane, path: item.path, name: item.name }));
    event.dataTransfer.effectAllowed = 'copy';
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function onDrop(event: DragEvent, targetPane: 'local' | 'remote') {
    event.preventDefault();
    if (!event.dataTransfer) return;
    const raw = event.dataTransfer.getData('text/plain');
    if (!raw) return;

    try {
      const parsed = JSON.parse(raw);
      if (parsed.pane === targetPane) return; // Same pane drag-and-drop ignored for now

      if (parsed.pane === 'local' && targetPane === 'remote') {
        const remoteDest = joinPath(remotePath, parsed.name);
        enqueueTransfer('upload', parsed.path, remoteDest);
        notifySuccess(t('sftp.queuedOneUpload', { name: parsed.name }));
      } else if (parsed.pane === 'remote' && targetPane === 'local') {
        const localDest = joinPath(localPath, parsed.name);
        enqueueTransfer('download', parsed.path, localDest);
        notifySuccess(t('sftp.queuedOneDownload', { name: parsed.name }));
      }
    } catch {}
  }

  // --- KEYBOARD SHORTCUTS ---
  function handleKeydown(e: KeyboardEvent) {
    // Typing in a field (path, search, rename…) or holding a modifier is never a file command:
    // Delete in the path box must delete text, not open "delete file".
    if (isTypingTarget(e.target) || e.ctrlKey || e.altKey || e.metaKey) return;
    // Nor while a dialog is open.
    if (
      showNewFolderModal ||
      showRenameModal ||
      showDeleteModal ||
      showEditorModal ||
      showChmodModal ||
      showCompressModal ||
      showExtractModal
    ) {
      return;
    }

    if (e.key === 'F5') {
      e.preventDefault();
      handleCopyTransfer();
    } else if (e.key === 'F6') {
      e.preventDefault();
      handleCopyTransfer(); // Move parity
    } else if (e.key === 'F7') {
      e.preventDefault();
      openNewFolderModal(activePane);
    } else if (e.key === 'F8' || e.key === 'Delete') {
      e.preventDefault();
      openDeleteModal();
    } else if (e.key === 'F2') {
      e.preventDefault();
      openRenameModal();
    } else if (e.key === 'F4') {
      e.preventDefault();
      openEditorModal();
    }
  }

  onMount(async () => {
    window.addEventListener('keydown', handleKeydown);

    unlistenProgress = await listen<SftpProgressPayload>('sftp-progress', (event) => {
      const payload = event.payload;
      const idx = transfers.findIndex((t) => t.id === payload.transfer_id);
      if (idx !== -1) {
        transfers[idx].bytesTransferred = payload.bytes_transferred;
        transfers[idx].totalBytes = payload.total_bytes;
        transfers[idx].speedBps = payload.speed_bps;
      }
    });

    await loadHosts();
    await fetchLocalFiles();
    if (currentHostId) {
      await fetchRemoteFiles();
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    if (unlistenProgress) {
      unlistenProgress();
    }
  });

  $effect(() => {
    const hostParam = page.url.searchParams.get('host');
    if (hostParam && hostParam !== currentHostId && hosts.some((h) => h.id === hostParam)) {
      currentHostId = hostParam;
      fetchRemoteFiles();
    }
  });
</script>

<div class="h-full flex flex-col">
<PageHeader
  icon="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
  accent="emerald"
  title={t('sftp.title')}
  subtitle={t('sftp.subtitle')}
/>
<PageContainer noPadding class="flex-1">
<div class="h-full flex flex-col space-y-3 p-3 text-neutral-800 dark:text-neutral-200">
  <!-- Top Bar: Host Selector & Controls -->
  <div class="flex flex-wrap items-center justify-between gap-3 bg-white dark:bg-[#181c22] p-3 rounded-lg border border-neutral-200 dark:border-slate-800 shadow-xs">
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold uppercase text-neutral-500 dark:text-slate-400">{t('sftp.remoteHost')}</span>
        <select
          bind:value={currentHostId}
          onchange={() => fetchRemoteFiles()}
          class="bg-neutral-50 dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2.5 py-1 text-sm text-neutral-800 dark:text-slate-200 focus:outline-none focus:border-cyan-500"
        >
          {#each hosts as h}
            <option value={h.id}>{h.label} ({h.username}@{h.address}:{h.port})</option>
          {/each}
        </select>
        {#if currentHost}
          <div class="flex items-center gap-1.5 ml-1">
            <Badge variant="info" size="xs">
              {activeProtocolBadge}
            </Badge>
            <select
              value={currentHost.protocol || 'ssh'}
              onchange={(e) => switchCurrentProtocol((e.currentTarget as HTMLSelectElement).value as ConnectionProtocol)}
              class="bg-neutral-50 dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2 py-0.5 text-xs text-neutral-700 dark:text-slate-300 focus:outline-none focus:border-cyan-500"
              title="Ganti Protokol VFS Host"
            >
              <option value="ssh">SSH / SFTP</option>
              <option value="scp">SCP</option>
              <option value="ftp">FTP</option>
              <option value="ftps">FTPS</option>
              <option value="webdav">WebDAV</option>
              <option value="s3">Amazon S3 / R2 / MinIO</option>
            </select>
          </div>
        {/if}
        <Button
          variant={showProtocolSelector ? 'primary' : 'secondary'}
          size="sm"
          onclick={() => (showProtocolSelector = !showProtocolSelector)}
          title="Koneksi Cepat VFS / Ganti Sumber"
        >
          <span class="flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
            Quick Connect
          </span>
        </Button>
      </div>

      <div class="flex items-center border border-neutral-300 dark:border-slate-700 rounded bg-neutral-100 dark:bg-slate-900 p-0.5 text-xs">
        <Button
          variant={viewMode === 'dual' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => (viewMode = 'dual')}
        >
          {t('sftp.dualPane')}
        </Button>
        <Button
          variant={viewMode === 'single' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => (viewMode = 'single')}
        >
          {t('sftp.remoteOnly')}
        </Button>
      </div>
    </div>

    <!-- Quick Action Hotkeys Reference -->
    <div class="hidden lg:flex items-center gap-2 text-xs text-neutral-500 dark:text-slate-400">
      <Badge variant="neutral" size="xs">F5</Badge> {t('sftp.hkCopy')}
      <Badge variant="neutral" size="xs">F7</Badge> {t('sftp.hkNewFolder')}
      <Badge variant="neutral" size="xs">F8</Badge> {t('sftp.hkDelete')}
      <Badge variant="neutral" size="xs">F2</Badge> {t('sftp.hkRename')}
      <Badge variant="neutral" size="xs">F4</Badge> {t('sftp.hkEdit')}
    </div>

    {#if currentHostId}
      <a
        href={`/session?host=${currentHostId}`}
        class="flex items-center gap-1.5 text-xs bg-neutral-100 dark:bg-slate-800 hover:bg-neutral-200 dark:hover:bg-slate-700 text-cyan-600 dark:text-cyan-400 px-2.5 py-1.5 rounded border border-neutral-300 dark:border-slate-700 transition"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        {t('sftp.terminalHere')}
      </a>
    {/if}
  </div>

  <!-- Error / Success Banners -->
  {#if errorMsg}
    <div class="bg-red-50 dark:bg-red-950/80 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 text-xs px-3 py-2 rounded flex justify-between items-center">
      <span>{errorMsg}</span>
      <Button variant="ghost" size="icon" onclick={() => (errorMsg = '')} class="h-5 w-5 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-200" aria-label={t('sftp.dismiss')}>
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/></svg>
      </Button>
    </div>
  {/if}
  {#if successMsg}
    <div class="bg-emerald-50 dark:bg-emerald-950/80 border border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300 text-xs px-3 py-2 rounded flex justify-between items-center">
      <span>{successMsg}</span>
      <Button variant="ghost" size="icon" onclick={() => (successMsg = '')} class="h-5 w-5 text-emerald-500 hover:text-emerald-700 dark:text-emerald-400 dark:hover:text-emerald-200" aria-label={t('sftp.dismiss')}>
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/></svg>
      </Button>
    </div>
  {/if}

  <!-- Quick Connect / Protocol Selector Drawer -->
  {#if showProtocolSelector}
    <div class="p-3.5 bg-neutral-50 dark:bg-[#1a1f26] border border-cyan-500/40 rounded-lg shadow-sm space-y-3 text-xs">
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-slate-800 pb-2">
        <div class="flex items-center gap-2">
          <span class="font-bold text-neutral-900 dark:text-slate-100 flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
            Unified VFS Quick Connect
          </span>
          <span class="text-neutral-500 dark:text-slate-400 text-[11px]">(SFTP, S3, WebDAV, FTP/FTPS)</span>
        </div>
        <Button
          variant="ghost"
          size="icon"
          onclick={() => (showProtocolSelector = false)}
          class="h-6 w-6 text-neutral-400 hover:text-neutral-600 dark:hover:text-slate-200"
          aria-label="Close Quick Connect"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/></svg>
        </Button>
      </div>

      {#if qcError}
        <div class="p-2 bg-red-50 dark:bg-red-950/60 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
          {qcError}
        </div>
      {/if}

      <!-- Protocol Tabs -->
      <div class="flex flex-wrap gap-1.5 bg-neutral-100 dark:bg-slate-900 p-1 rounded border border-neutral-200 dark:border-slate-800">
        <Button
          variant={qcProtocol === 'ssh' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => { qcProtocol = 'ssh'; qcPort = 22; }}
        >
          SSH / SFTP
        </Button>
        <Button
          variant={qcProtocol === 's3' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => { qcProtocol = 's3'; qcPort = 443; }}
        >
          Amazon S3 / R2 / MinIO
        </Button>
        <Button
          variant={qcProtocol === 'webdav' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => { qcProtocol = 'webdav'; qcPort = 443; }}
        >
          WebDAV
        </Button>
        <Button
          variant={qcProtocol === 'ftp' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => { qcProtocol = 'ftp'; qcPort = 21; }}
        >
          FTP
        </Button>
        <Button
          variant={qcProtocol === 'ftps' ? 'primary' : 'ghost'}
          size="sm"
          onclick={() => { qcProtocol = 'ftps'; qcPort = 21; }}
        >
          FTPS
        </Button>
      </div>

      <!-- Credential Fields -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-2.5">
        <div class="space-y-1 md:col-span-1">
          <Input
            id="qc-label"
            size="sm"
            label="Label"
            bind:value={qcLabel}
            placeholder={qcProtocol.toUpperCase() + ' Connection'}
          />
        </div>

        <div class="space-y-1 md:col-span-2">
          <Input
            id="qc-address"
            size="sm"
            label={qcProtocol === 's3'
              ? 'Endpoint URL / S3 Host (e.g. s3.amazonaws.com or https://minio.local:9000)'
              : qcProtocol === 'webdav'
                ? 'WebDAV URL (e.g. https://cloud.example.com/remote.php/dav/files/user/)'
                : 'Host / Address'}
            bind:value={qcAddress}
            placeholder={qcProtocol === 's3' ? 's3.amazonaws.com' : qcProtocol === 'webdav' ? 'https://dav.mailbox.org/servlet/webdav.infostore' : '192.168.1.10'}
          />
        </div>

        <div class="space-y-1 md:col-span-1">
          <label for="qc-port" class="block text-[11px] font-semibold text-neutral-600 dark:text-slate-400">
            Port
          </label>
          <input
            id="qc-port"
            type="number"
            bind:value={qcPort}
            class="w-full bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2.5 py-1 text-xs font-mono text-neutral-800 dark:text-slate-200 focus:outline-none focus:border-cyan-500"
          />
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-2.5">
        <div class="space-y-1">
          <Input
            id="qc-username"
            size="sm"
            label={qcProtocol === 's3' ? 'Access Key ID' : 'Username'}
            bind:value={qcUsername}
            placeholder={qcProtocol === 's3' ? '«redacted:AKIA…»' : 'admin'}
          />
        </div>

        <div class="space-y-1">
          <Input
            id="qc-password"
            size="sm"
            type="password"
            label={qcProtocol === 's3' ? 'Secret Access Key' : (qcProtocol === 'webdav' ? 'Password / Token' : 'Password')}
            bind:value={qcPassword}
            placeholder="••••••••••••"
          />
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-1">
        <Button
          type="button"
          variant="secondary"
          size="sm"
          onclick={() => (showProtocolSelector = false)}
        >
          {t('common.cancel')}
        </Button>
        <Button
          type="button"
          variant="primary"
          size="sm"
          disabled={qcConnecting || !qcAddress.trim()}
          loading={qcConnecting}
          onclick={handleQuickConnect}
        >
          Connect VFS
        </Button>
      </div>
    </div>
  {/if}

  <!-- Main Panes Area -->
  <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-3 min-h-0">
    <!-- LEFT PANE: Local File System -->
    {#if viewMode === 'dual'}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class={`flex flex-col bg-white dark:bg-[#181c22] rounded-lg border overflow-hidden transition ${
          activePane === 'local' ? 'border-cyan-500/80 ring-1 ring-cyan-500/40' : 'border-neutral-200 dark:border-slate-800'
        }`}
        onclick={() => (activePane = 'local')}
        role="region"
        aria-label={t('sftp.localFs')}
        ondragover={onDragOver}
        ondrop={(e) => onDrop(e, 'local')}
      >
        <!-- Local Toolbar & Path -->
        <div class="p-2 bg-neutral-50 dark:bg-[#1e232a] border-b border-neutral-200 dark:border-slate-800 flex items-center justify-between gap-2">
          <div class="flex items-center gap-1.5">
            <span class="text-xs font-bold text-amber-600 dark:text-amber-400">{t('sftp.local')}</span>
            <Button
              variant="ghost"
              size="icon"
              onclick={goUpLocal}
              title={t('sftp.parentDir')}
              aria-label={t('sftp.parentDir')}
              class="h-6 w-6"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
              </svg>
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onclick={() => navigateLocal('~')}
              title={t('sftp.homeDir')}
              aria-label={t('sftp.homeDir')}
              class="h-6 w-6"
            >
              ~
            </Button>
          </div>
          <Input
            variant="outline"
            size="sm"
            bind:value={localPath}
            onkeydown={(e) => e.key === 'Enter' && fetchLocalFiles()}
            class="flex-1 font-mono"
          />
          <Button
            variant="secondary"
            size="sm"
            onclick={() => openNewFolderModal('local')}
            title={t('sftp.newFolder')}
          >
            {t('sftp.addDir')}
          </Button>
        </div>

        <!-- Local Interactive Breadcrumbs -->
        {#if localBreadcrumbs().length > 0}
          <div class="px-3 py-1 bg-neutral-50 dark:bg-slate-900/60 border-b border-neutral-200 dark:border-slate-800/80 flex items-center gap-1 overflow-x-auto text-[11px] font-mono scrollbar-none">
            <button
              type="button"
              onclick={() => navigateLocal('/')}
              class="hover:text-rose-500 text-neutral-400 dark:text-slate-500 transition-colors shrink-0"
              title="Root"
            >
              /
            </button>
            {#each localBreadcrumbs() as seg, idx}
              <span class="text-neutral-300 dark:text-slate-600 shrink-0 select-none">/</span>
              <button
                type="button"
                onclick={() => navigateToBreadcrumbLocal(idx)}
                class="hover:text-rose-400 hover:underline text-neutral-600 dark:text-slate-300 transition-colors shrink-0 max-w-[140px] truncate"
              >
                {seg}
              </button>
            {/each}
          </div>
        {/if}

        <!-- Local File Table -->
        <div class="flex-1 overflow-auto text-xs font-mono" oncontextmenu={(e) => e.preventDefault()}>
          <table class="w-full border-collapse">
            <thead class="sticky top-0 bg-neutral-100/95 dark:bg-slate-900/95 text-neutral-600 dark:text-slate-400 border-b border-neutral-200 dark:border-slate-800 select-none">
              <tr>
                <th class="text-left py-1.5 px-3">{t('sftp.colName')}</th>
                <th class="text-right py-1.5 px-3">{t('sftp.colSize')}</th>
                <th class="text-right py-1.5 px-3">{t('sftp.colModified')}</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-200 dark:divide-slate-800/40">
              {#if localLoading}
                <tr>
                  <td colspan="3" class="text-center py-6 text-neutral-400 dark:text-slate-500">{t('sftp.loadingLocal')}</td>
                </tr>
              {:else if localFiles.length === 0}
                <tr>
                  <td colspan="3" class="text-center py-6 text-neutral-400 dark:text-slate-500">{t('sftp.emptyDir')}</td>
                </tr>
              {:else}
                {#each localFiles as item}
                  <tr
                    class={`cursor-pointer select-none transition ${
                      localSelectedPaths.has(item.path) || localLastSelected?.path === item.path
                        ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white'
                        : 'hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300'
                    }`}
                    onclick={() => toggleLocalSelect(item)}
                    ondblclick={() => item.is_dir ? navigateLocal(item.path) : openEditorModal()}
                    draggable="true"
                    ondragstart={(e) => onDragStart(e, 'local', item)}
                    oncontextmenu={(e) => openLocalCtxMenu(e, item)}
                  >
                    <td class="py-1.5 px-3 flex items-center gap-2 truncate max-w-[200px]">
                      {#if item.is_dir}
                        <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                      {:else}
                        <svg class="w-4 h-4 shrink-0 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                      {/if}
                      <span class="truncate">{item.name}</span>
                    </td>
                    <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">
                      {item.is_dir ? '<DIR>' : formatSize(item.size)}
                    </td>
                    <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">
                      {formatMtime(item.mtime)}
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    {/if}

    <!-- RIGHT PANE: Remote SFTP Server -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class={`flex flex-col bg-white dark:bg-[#181c22] rounded-lg border overflow-hidden transition ${
        viewMode === 'single' ? 'col-span-1 md:col-span-2' : ''
      } ${activePane === 'remote' ? 'border-cyan-500/80 ring-1 ring-cyan-500/40' : 'border-neutral-200 dark:border-slate-800'}`}
      onclick={() => (activePane = 'remote')}
      role="region"
      aria-label={t('sftp.remoteServer')}
      ondragover={onDragOver}
      ondrop={(e) => onDrop(e, 'remote')}
    >
      <!-- Remote Toolbar & Path -->
      <div class="p-2 bg-neutral-50 dark:bg-[#1e232a] border-b border-neutral-200 dark:border-slate-800 flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5">
          <span class="text-xs font-bold text-cyan-600 dark:text-cyan-400">{t('sftp.remote')}</span>
          <Button
            variant="ghost"
            size="icon"
            onclick={goUpRemote}
            title={t('sftp.parentDir')}
            aria-label={t('sftp.parentDir')}
            class="h-6 w-6"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18" />
            </svg>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onclick={() => navigateRemote('/')}
            title={t('sftp.rootDir')}
            aria-label={t('sftp.rootDir')}
            class="h-6 w-6"
          >
            /
          </Button>
        </div>
        <Input
          variant="outline"
          size="sm"
          bind:value={remotePath}
          onkeydown={(e) => e.key === 'Enter' && fetchRemoteFiles()}
          class="flex-1 font-mono"
        />
        <Button
          variant="secondary"
          size="sm"
          onclick={() => openNewFolderModal('remote')}
          title={t('sftp.newFolder')}
        >
          {t('sftp.addDir')}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onclick={openCompressModal}
          title={t('sftp.compressSelected')}
        >
          {t('sftp.zip')}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onclick={() => (showSyncModal = true)}
          title={t('sftp.syncTitle')}
        >
          {t('sftp.sync')}
        </Button>
        <Button
          variant={showRemoteSearch ? 'primary' : 'secondary'}
          size="sm"
          onclick={() => (showRemoteSearch = !showRemoteSearch)}
          title={t('sftp.searchTitle')}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {t('sftp.search')}
        </Button>
      </div>

      <!-- Remote Interactive Breadcrumbs -->
      {#if remoteBreadcrumbs().length > 0}
        <div class="px-3 py-1 bg-neutral-50 dark:bg-slate-900/60 border-b border-neutral-200 dark:border-slate-800/80 flex items-center gap-1 overflow-x-auto text-[11px] font-mono scrollbar-none">
          <button
            type="button"
            onclick={() => navigateRemote('/')}
            class="hover:text-cyan-500 text-neutral-400 dark:text-slate-500 transition-colors shrink-0"
            title="Root"
          >
            /
          </button>
          {#each remoteBreadcrumbs() as seg, idx}
            <span class="text-neutral-300 dark:text-slate-600 shrink-0 select-none">/</span>
            <button
              type="button"
              onclick={() => navigateToBreadcrumbRemote(idx)}
              class="hover:text-cyan-400 hover:underline text-neutral-600 dark:text-slate-300 transition-colors shrink-0 max-w-[140px] truncate"
            >
              {seg}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Remote Search & Filter Drawer -->
      {#if showRemoteSearch}
        <div class="p-2.5 bg-neutral-100/90 dark:bg-[#151921] border-b border-neutral-200 dark:border-slate-800 flex flex-col gap-2 text-xs">
          <div class="flex items-center gap-2">
            <Input
              variant="outline"
              size="sm"
              placeholder={t('sftp.searchPlaceholder')}
              bind:value={remoteSearchQuery}
              onkeydown={(e) => e.key === "Enter" && executeRemoteSearch()}
              class="flex-1 font-mono"
            />
            <Button
              variant="primary"
              size="sm"
              onclick={executeRemoteSearch}
              disabled={isSearchingRemote || !remoteSearchQuery.trim()}
              loading={isSearchingRemote}
            >
              {t('sftp.find')}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onclick={() => { showRemoteSearch = false; remoteSearchResults = []; }}
              title={t('sftp.closeSearch')}
              aria-label={t('sftp.closeSearch')}
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/></svg>
            </Button>
          </div>
          <div class="flex items-center gap-3 text-neutral-600 dark:text-slate-400">
            <span class="text-[11px] font-semibold">{t('sftp.filters')}</span>
            <label class="flex items-center gap-1 text-[11px]">
              {t('sftp.minKb')}
              <input
                type="number"
                min="0"
                placeholder="0"
                bind:value={searchMinSizeKB}
                class="w-16 bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-1.5 py-0.5 text-xs font-mono"
              />
            </label>
            <label class="flex items-center gap-1 text-[11px]">
              {t('sftp.maxKb')}
              <input
                type="number"
                min="0"
                placeholder="∞"
                bind:value={searchMaxSizeKB}
                class="w-16 bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-1.5 py-0.5 text-xs font-mono"
              />
            </label>
            <span class="text-[10px] text-neutral-400 dark:text-slate-500 ml-auto">{t('sftp.under', { path: remotePath })}</span>
          </div>

          {#if remoteSearchResults.length > 0}
            <div class="max-h-48 overflow-auto border border-neutral-200 dark:border-slate-800 rounded bg-white dark:bg-slate-900 divide-y divide-neutral-100 dark:divide-slate-800/60 font-mono text-[11px]">
              {#each remoteSearchResults as result}
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => jumpToSearchResult(result)}
                  onkeydown={(e) => e.key === "Enter" && jumpToSearchResult(result)}
                  class="p-1.5 px-2 hover:bg-neutral-100 dark:hover:bg-neutral-800/60 cursor-pointer flex items-center justify-between gap-2"
                >
                  <div class="flex items-center gap-1.5 truncate">
                    {#if result.is_dir}
                      <svg class="w-3.5 h-3.5 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                    {:else}
                      <svg class="w-3.5 h-3.5 shrink-0 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                    {/if}
                    <span class="font-medium text-neutral-800 dark:text-slate-200 truncate">{result.name}</span>
                    <span class="text-neutral-400 dark:text-slate-500 text-[10px] truncate">({result.path})</span>
                  </div>
                  <div class="shrink-0 text-neutral-500 dark:text-slate-400 text-[10px]">
                    {result.is_dir ? "<DIR>" : formatSize(result.size)}
                  </div>
                </div>
              {/each}
            </div>
          {:else if !isSearchingRemote && remoteSearchQuery && remoteSearchResults.length === 0}
            <div class="text-[11px] text-neutral-400 dark:text-slate-500 italic py-1">{t('sftp.noMatches')}</div>
          {/if}
        </div>
      {/if}

      <!-- Remote File Table -->
      <div class="flex-1 overflow-auto text-xs font-mono" oncontextmenu={(e) => e.preventDefault()}>
        <table class="w-full border-collapse">
          <thead class="sticky top-0 bg-neutral-100/95 dark:bg-slate-900/95 text-neutral-600 dark:text-slate-400 border-b border-neutral-200 dark:border-slate-800 select-none">
            <tr>
              <th class="text-left py-1.5 px-3">{t('sftp.colName')}</th>
              <th class="text-right py-1.5 px-3">{t('sftp.colSize')}</th>
              <th class="text-center py-1.5 px-3">{t('sftp.colRights')}</th>
              <th class="text-right py-1.5 px-3">{t('sftp.colModified')}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-neutral-200 dark:divide-slate-800/40">
            {#if remoteLoading}
              <tr>
                <td colspan="4" class="text-center py-6 text-neutral-400 dark:text-slate-500">{t('sftp.loadingRemote')}</td>
              </tr>
            {:else if remoteFiles.length === 0}
              <tr>
                <td colspan="4" class="text-center py-6 text-neutral-400 dark:text-slate-500">{t('sftp.emptyDir')}</td>
              </tr>
            {:else}
              {#each remoteFiles as item}
                <tr
                  class={`cursor-pointer select-none transition ${
                    remoteSelectedPaths.has(item.path) || remoteLastSelected?.path === item.path
                      ? 'bg-neutral-200/70 dark:bg-neutral-800 text-neutral-900 dark:text-white'
                      : 'hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300'
                  }`}
                  onclick={() => toggleRemoteSelect(item)}
                  ondblclick={() => item.is_dir ? navigateRemote(item.path) : openEditorModal()}
                  draggable="true"
                  ondragstart={(e) => onDragStart(e, 'remote', item)}
                  oncontextmenu={(e) => openRemoteCtxMenu(e, item)}
                >
                  <td class="py-1.5 px-3 flex items-center gap-2 truncate max-w-[200px]">
                    {#if item.is_dir}
                      <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
                    {:else}
                      <svg class="w-4 h-4 shrink-0 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
                    {/if}
                    <span class="truncate">{item.name}</span>
                  </td>
                  <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">
                    {item.is_dir ? '<DIR>' : formatSize(item.size)}
                  </td>
                  <td
                    class="py-1.5 px-3 text-center text-neutral-500 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-300 cursor-pointer whitespace-nowrap"
                    onclick={(e) => { e.stopPropagation(); openChmodModal(item); }}
                  >
                    {formatPermissions(item.mode)}
                  </td>
                  <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">
                    {formatMtime(item.mtime)}
                  </td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <!-- Bottom Panel: Transfer Queue Manager -->
  <div class="h-36 bg-white dark:bg-[#181c22] rounded-lg border border-neutral-200 dark:border-slate-800 flex flex-col overflow-hidden text-xs shadow-xs">
    <div class="bg-neutral-50 dark:bg-[#1e232a] px-3 py-1.5 border-b border-neutral-200 dark:border-slate-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="font-bold text-neutral-700 dark:text-slate-300">{t('sftp.queueTitle')}</span>
        <span class="text-neutral-500 dark:text-slate-500">
          {t('sftp.queueCounts', { active: transfers.filter((tr) => tr.status === 'active').length, queued: transfers.filter((tr) => tr.status === 'queued').length })}
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          onclick={clearCompletedTransfers}
          class="text-neutral-600 hover:text-neutral-800 dark:text-slate-400 dark:hover:text-slate-200 px-2 py-0.5"
        >
          {t('sftp.clearFinished')}
        </Button>
      </div>
    </div>

    <div class="flex-1 overflow-auto divide-y divide-neutral-200 dark:divide-slate-800/40 font-mono">
      {#if transfers.length === 0}
        <div class="text-center py-6 text-neutral-400 dark:text-slate-500">{t('sftp.queueEmpty')}</div>
      {:else}
        {#each transfers as item}
          <div class="p-2 flex items-center justify-between gap-3 hover:bg-neutral-50 dark:hover:bg-slate-800/30">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 truncate">
                <span class={`font-bold ${item.direction === 'upload' ? 'text-cyan-600 dark:text-cyan-400' : 'text-emerald-600 dark:text-emerald-400'}`}>
                  {item.direction === 'upload' ? t('sftp.upload') : t('sftp.download')}
                </span>
                <span class="text-neutral-700 dark:text-slate-300 truncate">{item.source.split('/').pop()}</span>
                <span class="text-neutral-400 dark:text-slate-500">→</span>
                <span class="text-neutral-500 dark:text-slate-400 truncate">{item.target}</span>
              </div>

              <!-- Progress Bar -->
              <div class="mt-1 flex items-center gap-2">
                <div class="flex-1 bg-neutral-200 dark:bg-slate-900 rounded-full h-1.5 overflow-hidden">
                  <div
                    class={`h-full transition-all duration-150 ${
                      item.status === 'completed'
                        ? 'bg-emerald-500'
                        : item.status === 'failed'
                        ? 'bg-red-500'
                        : item.status === 'cancelled'
                        ? 'bg-neutral-400 dark:bg-slate-600'
                        : 'bg-cyan-500'
                    }`}
                    style={`width: ${
                      item.totalBytes > 0
                        ? Math.min(100, (item.bytesTransferred / item.totalBytes) * 100)
                        : item.status === 'completed' ? 100 : 0
                    }%`}
                  ></div>
                </div>
                <span class="text-[10px] text-neutral-500 dark:text-slate-400 whitespace-nowrap">
                  {formatSize(item.bytesTransferred)} / {formatSize(item.totalBytes)}
                </span>
                {#if item.status === 'active' && item.speedBps > 0}
                  <span class="text-[10px] text-cyan-600 dark:text-cyan-400 whitespace-nowrap">
                    ({formatSpeed(item.speedBps)})
                  </span>
                {/if}
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Badge
                variant={item.status === 'completed' ? 'success'
                  : item.status === 'failed' ? 'danger'
                  : item.status === 'active' ? 'info'
                  : 'neutral'}
                size="xs"
              >
                {t(`sftp.status.${item.status}`)}
              </Badge>
              {#if item.status === 'active'}
                <Button
                  variant="danger"
                  size="sm"
                  onclick={() => cancelQueueItem(item.id)}
                  title={t('sftp.cancelTransfer')}
                  aria-label={t('sftp.cancelTransfer')}
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/></svg>
                </Button>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<Modal
  bind:open={showNewFileModal}
  title={t('sftp.newFileTitle', { pane: newFileTargetPane === 'local' ? t('sftp.paneLocal') : t('sftp.paneRemote') })}
  size="sm"
>
  <Input
    bind:value={newFileName}
    placeholder="filename.txt"
    onkeydown={(e) => e.key === 'Enter' && confirmNewFile()}
    class="text-xs"
  />
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showNewFileModal = false)}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirmNewFile}
    >
      {t('common.create')}
    </Button>
  {/snippet}
</Modal>

<Modal
  bind:open={showNewFolderModal}
  title={t('sftp.newFolderTitle', { pane: newFolderTargetPane === 'local' ? t('sftp.paneLocal') : t('sftp.paneRemote') })}
  size="sm"
>
  <Input
    bind:value={newFolderName}
    placeholder={t('sftp.folderNamePlaceholder')}
    onkeydown={(e) => e.key === 'Enter' && confirmNewFolder()}
    class="text-xs"
  />
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showNewFolderModal = false)}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirmNewFolder}
    >
      {t('common.create')}
    </Button>
  {/snippet}
</Modal>

<Modal
  bind:open={showRenameModal}
  title={t('sftp.renameTitle')}
  size="sm"
>
  <Input
    bind:value={renameNewName}
    onkeydown={(e) => e.key === 'Enter' && confirmRename()}
    class="text-xs"
  />
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showRenameModal = false)}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirmRename}
      disabled={!renameItem}
    >
      {t('common.rename')}
    </Button>
  {/snippet}
</Modal>

<Modal
  bind:open={showDeleteModal}
  title={t('sftp.deleteTitle')}
  description={deleteTarget ? t('sftp.deleteBody', { count: deleteTarget.paths.length, where: deleteTarget.pane === 'local' ? t('sftp.localFsShort') : t('sftp.remoteSftpShort') }) : ''}
  size="sm"
>
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showDeleteModal = false)}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="danger"
      size="sm"
      onclick={confirmDelete}
      disabled={!deleteTarget}
    >
      {t('common.delete')}
    </Button>
  {/snippet}
</Modal>

<Modal
  bind:open={showEditorModal}
  title={editorItem ? `${t('sftp.editor')} ${editorItem.name} (${editorItem.pane === 'local' ? t('sftp.paneLocal') : t('sftp.paneRemote')})` : ''}
  size="xl"
  closeOnBackdrop={false}
>
  {#if RemoteFileEditor && editorItem}
    <RemoteFileEditor
      bind:content={editorContent}
      filename={editorItem.name}
      pane={editorItem.pane}
      onSave={saveEditorFile}
      saving={isEditorSaving}
      loading={isEditorLoading}
    />
  {:else}
    <div class="flex-1 flex items-center justify-center text-xs text-neutral-500">{t('editor.loading')}</div>
  {/if}
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showEditorModal = false)}
    >
      {t('common.close')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={saveEditorFile}
      disabled={isEditorSaving || isEditorLoading || !editorItem}
      loading={isEditorSaving}
    >
      {isEditorSaving ? t('sftp.saving') : t('common.save')}
    </Button>
  {/snippet}
</Modal>

<Modal
  bind:open={showChmodModal}
  title={chmodTarget ? `${t('sftp.permissionsFor')} ${chmodTarget.name}` : ''}
  size="md"
>
  <div class="text-xs space-y-4">
    <div class="grid grid-cols-3 gap-3 bg-neutral-50 dark:bg-slate-900 p-3 rounded border border-neutral-200 dark:border-slate-800">
      <!-- User -->
      <div class="space-y-1.5">
        <span class="font-bold text-neutral-700 dark:text-slate-400">{t('sftp.owner')}</span>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodUserR} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.read')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodUserW} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.write')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodUserX} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.execute')}
        </label>
      </div>

      <!-- Group -->
      <div class="space-y-1.5">
        <span class="font-bold text-neutral-700 dark:text-slate-400">{t('sftp.group')}</span>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodGroupR} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.read')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodGroupW} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.write')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodGroupX} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.execute')}
        </label>
      </div>

      <!-- Others -->
      <div class="space-y-1.5">
        <span class="font-bold text-neutral-700 dark:text-slate-400">{t('sftp.others')}</span>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodOtherR} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.read')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodOtherW} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.write')}
        </label>
        <label class="flex items-center gap-1.5 text-neutral-700 dark:text-slate-300 cursor-pointer">
          <input type="checkbox" bind:checked={chmodOtherX} onchange={updateChmodOctalFromCheckboxes} /> {t('sftp.execute')}
        </label>
      </div>
    </div>

    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="font-bold text-neutral-700 dark:text-slate-400">{t('sftp.octal')}</span>
        <input
          type="text"
          bind:value={chmodOctal}
          oninput={() => updateChmodCheckboxesFromOctal(chmodOctal)}
          maxlength="4"
          class="w-20 bg-neutral-100 dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2 py-1 text-center font-mono text-cyan-600 dark:text-cyan-400 focus:outline-none"
        />
      </div>
      <div class="flex gap-2">
        <Button
          variant="secondary"
          size="sm"
          onclick={() => (showChmodModal = false)}
        >
          {t('common.cancel')}
        </Button>
        <Button
          variant="primary"
          size="sm"
          onclick={confirmChmod}
          disabled={!chmodTarget}
        >
          {t('sftp.apply')}
        </Button>
      </div>
    </div>
  </div>
</Modal>

<!-- CONTEXT MENU -->
{#if ctxMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="fixed inset-0 z-50"
    onclick={closeCtxMenu}
    oncontextmenu={(e) => { e.preventDefault(); closeCtxMenu(); }}
    role="presentation"
  >
    <div
      class="fixed bg-white dark:bg-[#1a1f29] border border-neutral-200 dark:border-white/10 rounded-lg shadow-2xl py-1 z-50 text-xs min-w-[180px] font-sans"
      style={`top: ${Math.min(ctxMenu.y, 400)}px; left: ${Math.min(ctxMenu.x, 800)}px;`}
      onclick={(e) => e.stopPropagation()}
      role="menu"
      tabindex="-1"
    >
      {#if ctxMenu.pane === 'remote'}
        <!-- Remote Actions -->
        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); handleCopyTransfer(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
            {t('sftp.ctxDownload')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F5</span>
        </Button>

        {#if !ctxMenu.item.is_dir}
          <Button
            variant="ghost"
            size="sm"
            class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
            onclick={() => { closeCtxMenu(); openEditorModal(); }}
          >
            <span class="flex items-center gap-2">
              <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"/></svg>
              {t('sftp.ctxEditView')}
            </span>
            <span class="text-[10px] text-neutral-400 font-mono">F4</span>
          </Button>
        {/if}

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openRenameModal(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M7 7h.01M7 3h5a1.99 1.99 0 011.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V5a2 2 0 012-2z"/></svg>
            {t('sftp.ctxRename')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F2</span>
        </Button>

        <Button
          variant="danger"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none border-0"
          onclick={() => { closeCtxMenu(); openDeleteModal(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            {t('sftp.ctxDelete')}
          </span>
          <span class="text-[10px] opacity-70 font-mono">Del</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { const item = ctxMenu!.item as SftpFileEntry; closeCtxMenu(); openChmodModal(item); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/></svg>
            {t('sftp.ctxPermissions')}
          </span>
        </Button>

        <div class="my-1 border-t border-neutral-200 dark:border-white/10"></div>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openNewFileModal('remote'); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
            {t('sftp.ctxNewFile')}
          </span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openNewFolderModal('remote'); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
            {t('sftp.ctxNewFolder')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F7</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => {
            const item = ctxMenu!.item;
            closeCtxMenu();
            navigator.clipboard.writeText(item.path);
            notifySuccess(t('sftp.copiedPath', { path: item.path }));
          }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/></svg>
            {t('sftp.ctxCopyPath')}
          </span>
        </Button>

        <div class="my-1 border-t border-neutral-200 dark:border-white/10"></div>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-start text-left px-3 py-1.5 font-normal rounded-none gap-2"
          onclick={() => { closeCtxMenu(); openCompressModal(); }}
        >
          <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/></svg>
          {t('sftp.ctxCompress')}
        </Button>

        {#if isArchive(ctxMenu.item.name)}
          <Button
            variant="ghost"
            size="sm"
            class="w-full !justify-start text-left px-3 py-1.5 font-normal rounded-none gap-2"
            onclick={() => { const item = ctxMenu!.item as SftpFileEntry; closeCtxMenu(); openExtractHere(item); }}
          >
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/></svg>
            {t('sftp.ctxExtractHere')}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            class="w-full !justify-start text-left px-3 py-1.5 font-normal rounded-none gap-2"
            onclick={() => { const item = ctxMenu!.item as SftpFileEntry; closeCtxMenu(); openExtractTo(item); }}
          >
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"/></svg>
            {t('sftp.ctxExtractTo')}
          </Button>
        {/if}

      {:else}
        <!-- Local Actions -->
        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); handleCopyTransfer(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/></svg>
            {t('sftp.ctxUpload')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F5</span>
        </Button>

        {#if !ctxMenu.item.is_dir}
          <Button
            variant="ghost"
            size="sm"
            class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
            onclick={() => { closeCtxMenu(); openEditorModal(); }}
          >
            <span class="flex items-center gap-2">
              <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"/></svg>
              {t('sftp.ctxEditView')}
            </span>
            <span class="text-[10px] text-neutral-400 font-mono">F4</span>
          </Button>
        {/if}

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openRenameModal(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M7 7h.01M7 3h5a1.99 1.99 0 011.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V5a2 2 0 012-2z"/></svg>
            {t('sftp.ctxRename')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F2</span>
        </Button>

        <Button
          variant="danger"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none border-0"
          onclick={() => { closeCtxMenu(); openDeleteModal(); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            {t('sftp.ctxDelete')}
          </span>
          <span class="text-[10px] opacity-70 font-mono">Del</span>
        </Button>

        <div class="my-1 border-t border-neutral-200 dark:border-white/10"></div>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openNewFileModal('local'); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
            {t('sftp.ctxNewFile')}
          </span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => { closeCtxMenu(); openNewFolderModal('local'); }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/></svg>
            {t('sftp.ctxNewFolder')}
          </span>
          <span class="text-[10px] text-neutral-400 font-mono">F7</span>
        </Button>

        <Button
          variant="ghost"
          size="sm"
          class="w-full !justify-between text-left px-3 py-1.5 font-normal rounded-none"
          onclick={() => {
            const item = ctxMenu!.item;
            closeCtxMenu();
            navigator.clipboard.writeText(item.path);
            notifySuccess(t('sftp.copiedPath', { path: item.path }));
          }}
        >
          <span class="flex items-center gap-2">
            <svg class="w-4 h-4 shrink-0 text-neutral-500 dark:text-neutral-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/></svg>
            {t('sftp.ctxCopyPath')}
          </span>
        </Button>
      {/if}
    </div>
  </div>
{/if}

<!-- MODAL: COMPRESS -->
<Modal
  bind:open={showCompressModal}
  title={t('sftp.compressTitle', { count: compressItems.length })}
  description={t('sftp.items', { list: compressItems.slice(0, 3).join(', ') + (compressItems.length > 3 ? '...' : '') })}
  size="sm"
>
  <div class="space-y-1">
    <Input
      id="archive-name-input"
      label={t('sftp.archiveName')}
      bind:value={compressArchiveName}
      placeholder="archive.tar.gz"
      onkeydown={(e) => e.key === 'Enter' && confirmCompress()}
      class="font-mono text-xs"
    />
  </div>
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showCompressModal = false)}
      disabled={isCompressing}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirmCompress}
      disabled={isCompressing || !compressArchiveName.trim()}
      loading={isCompressing}
    >
      {isCompressing ? t('sftp.compressing') : t('sftp.compress')}
    </Button>
  {/snippet}
</Modal>

<!-- MODAL: EXTRACT -->
<Modal
  bind:open={showExtractModal}
  title={t('sftp.extractTitle')}
  size="md"
>
  <div class="space-y-3">
    <p class="text-xs text-neutral-500 dark:text-slate-400 truncate">
      {t('sftp.source')} <span class="font-mono text-cyan-600 dark:text-cyan-400">{extractArchivePath.split('/').pop()}</span>
    </p>
    <Input
      id="extract-dest-input"
      label={t('sftp.destination')}
      bind:value={extractDestDir}
      onkeydown={(e) => e.key === 'Enter' && confirmExtract()}
      class="font-mono text-xs"
    />
  </div>
  {#snippet footer()}
    <Button
      variant="secondary"
      size="sm"
      onclick={() => (showExtractModal = false)}
      disabled={isExtracting}
    >
      {t('common.cancel')}
    </Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirmExtract}
      disabled={isExtracting || !extractDestDir.trim()}
      loading={isExtracting}
    >
      {isExtracting ? t('sftp.extracting') : t('sftp.extract')}
    </Button>
  {/snippet}
</Modal>

<!-- MODAL: DIRECTORY SYNC & LIVE WATCH -->
<Modal
  bind:open={showSyncModal}
  size="xl"
  class="max-w-3xl"
>
  <ProGate title="Directory & Broadcast Sync Pro" description="Sinkronisasi direktori otomatis dan live watcher bidirectional memerlukan lisensi CATerm Pro.">
    <DirectorySync
      hostId={currentHostId}
      initialLocalPath={localPath}
      initialRemotePath={remotePath}
      onClose={() => (showSyncModal = false)}
    />
  </ProGate>
</Modal>
</PageContainer>
</div>

