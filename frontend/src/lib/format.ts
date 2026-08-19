export const CURRENCY_SYMBOL = import.meta.env.PUBLIC_CURRENCY_SYMBOL ?? "FCFA";

// Backend sends decimals as strings (e.g. "1.000", "25.000") to preserve
// precision - displayed raw, "1.000" reads as "one thousand" to anyone used
// to "." as a thousands separator. Format with fr-FR (comma decimal, space
// thousands) and trim the padding instead.
const quantityFormatter = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 3 });
const amountFormatter = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 2 });

export function formatQuantity(value: string | number): string {
  return quantityFormatter.format(Number(value));
}

export function formatAmount(value: string | number): string {
  return amountFormatter.format(Number(value));
}
