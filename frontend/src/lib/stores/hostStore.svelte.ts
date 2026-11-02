import type { HostRecord } from '$lib/api/hosts';
import { listHosts } from '$lib/api/hosts';
import type { GroupRecord } from '$lib/api/groups';
import { listGroups } from '$lib/api/groups';

let cachedHosts = $state<HostRecord[]>([]);
let cachedGroups = $state<GroupRecord[]>([]);
let isLoaded = $state<boolean>(false);
let isLoading = $state<boolean>(false);

export function getCachedHosts(): HostRecord[] {
  return cachedHosts;
}

export function getCachedGroups(): GroupRecord[] {
  return cachedGroups;
}

export function isHostsLoaded(): boolean {
  return isLoaded;
}

export function setCachedHosts(hosts: HostRecord[]) {
  cachedHosts = hosts;
  isLoaded = true;
}

export function setCachedGroups(groups: GroupRecord[]) {
  cachedGroups = groups;
}

export async function loadHostsAndGroupsCached(force = false): Promise<{ hosts: HostRecord[]; groups: GroupRecord[] }> {
  if (isLoaded && !force) {
    return { hosts: cachedHosts, groups: cachedGroups };
  }
  if (isLoading) {
    return { hosts: cachedHosts, groups: cachedGroups };
  }
  isLoading = true;
  try {
    const [hostsRes, groupsRes] = await Promise.all([listHosts(), listGroups()]);
    cachedHosts = hostsRes;
    cachedGroups = groupsRes;
    isLoaded = true;
    return { hosts: hostsRes, groups: groupsRes };
  } finally {
    isLoading = false;
  }
}
