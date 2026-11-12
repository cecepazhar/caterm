// CATerm Pro pricing as shown in the app — v5.1, mirrors
// `0 - Bussines Plan/08-pricing/01-master-pricing-strategy.md` (the single source of truth).
// These numbers are display only: the amounts actually charged live in the Creem checkout (the
// only payment gateway) and must be changed together with it, never on their own.
// Prices are USD only (v5.0): rupiah licences are sold by the owner on Shopee, outside the app.
// Founder All-Package (CATerm + CAMark + CAShell, 7 devices each): price follows the slot phase
// served by `GET /api/founder/slots`, never a hardcoded remaining count.
//
// AI is 100% BYOK (Bring Your Own Key): no plan includes an AI request quota.

export type Currency = 'USD';
/** License Key terms (1 device, offline HWID activation). */
export type LicenseTerm = 'lifetime' | 'yearly' | 'monthly';
/** Subscription billing intervals (5 devices, GCC cloud account). */
export type BillingInterval = 'monthly' | 'yearly';

export interface PricePoint {
  readonly normal: number;
  readonly promo: number;
}

export type DualPrice = Readonly<Record<Currency, PricePoint>>;

export const PRO_PRICING = {
  /**
   * Whether the promo (struck-through) prices are shown. Keep `false` unless the gateways are
   * charging the promo amounts right now (BFCM / launch campaign).
   */
  promoActive: false,
  license: {
    devices: 1,
    lifetime: { USD: { normal: 9.9, promo: 3.9 } },
    yearly: { USD: { normal: 6.9, promo: 2.9 } },
    monthly: { USD: { normal: 1.9, promo: 0.99 } }
  },
  subscription: {
    devices: 5,
    monthly: { USD: { normal: 10, promo: 4.9 } },
    yearly: { USD: { normal: 120, promo: 79 } },
    /** The monthly promo lasts this many months; from month 13 it returns to the normal price. */
    promoMonths: 12
  },
  /** Founder All-Package: lifetime, 3 apps x 7 devices, 100 slots across all channels. */
  founder: {
    apps: 3,
    devices: 7,
    slots: 100,
    flashSlots: 30,
    flashPrice: 129,
    regularPrice: 139,
    /** Struck-through valuation (master pricing §3.3). */
    anchor: 285
  },
  /** Checkout gateway per currency (Creem is the only gateway). */
  gateway: { USD: 'Creem' },
  trialDays: 7,
  /** One account covers up to this many people (owner + 6), at no extra cost. */
  maxMembers: 7,
  /** Devices a Pro subscription syncs across. */
  syncDevices: 5
} as const;

export const LICENSE_TERMS: readonly LicenseTerm[] = ['lifetime', 'yearly', 'monthly'];

/** `$10`, `$9.9`, `$0.99` — whole dollars without decimals, otherwise at most two decimals. */
export function formatUsd(amount: number): string {
  return `$${Number(amount.toFixed(2))}`;
}

export function formatPrice(amount: number, _currency: Currency = 'USD'): string {
  return formatUsd(amount);
}

/** Prices are USD only. */
export function defaultCurrency(): Currency {
  return 'USD';
}

/**
 * The price to display for one entry: `current` is what the customer pays now, `original` is the
 * struck-through normal price while a promo is active (otherwise `null`).
 */
export function displayPrice(
  price: DualPrice,
  currency: Currency,
  promoActive: boolean = PRO_PRICING.promoActive
): { current: string; original: string | null } {
  const { normal, promo } = price[currency];
  return promoActive
    ? { current: formatPrice(promo, currency), original: formatPrice(normal, currency) }
    : { current: formatPrice(normal, currency), original: null };
}
