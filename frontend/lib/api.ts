import type { ApiErrorResponse, DecodedInvoice } from "@/types/invoice";

const API_BASE_URL = (
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3001"
).replace(/\/$/, "");

const BOLT11_PREFIX = /^ln(?:bc|tb|bcrt|tbs)/i;

const ERROR_MESSAGES: Record<string, string> = {
  EMPTY_INVOICE: "Paste a BOLT11 invoice to continue.",
  INVALID_INVOICE:
    "That doesn’t look like a valid BOLT11 invoice. Check the value and try again.",
  INVOICE_TOO_LONG: "This invoice is too long to decode.",
  EXPIRY_OVERFLOW: "This invoice contains an unsupported expiry value.",
};

export class ApiRequestError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "ApiRequestError";
  }
}

export async function decodeInvoice(invoice: string): Promise<DecodedInvoice> {
  const normalizedInvoice = invoice.trim();

  if (!normalizedInvoice) {
    throw new ApiRequestError(ERROR_MESSAGES.EMPTY_INVOICE, "EMPTY_INVOICE", 400);
  }

  if (!BOLT11_PREFIX.test(normalizedInvoice)) {
    throw new ApiRequestError(
      "That doesn’t look like a BOLT11 invoice. Paste a Lightning invoice beginning with lnbc, lntb, lnbcrt, or lntbs.",
      "INVALID_INVOICE",
      400,
    );
  }

  let response: Response;

  try {
    response = await fetch(
      `${API_BASE_URL}/api/v1/invoices/decode/${encodeURIComponent(normalizedInvoice)}`,
      {
        headers: { Accept: "application/json" },
        cache: "no-store",
      },
    );
  } catch {
    throw new ApiRequestError(
      "The decoder service could not be reached. Please try again shortly.",
      "NETWORK_ERROR",
      0,
    );
  }

  if (!response.ok) {
    let payload: ApiErrorResponse | null = null;

    try {
      payload = (await response.json()) as ApiErrorResponse;
    } catch {
      // The server or an upstream proxy may return a non-JSON error page.
    }

    const code = payload?.error.code ?? "API_ERROR";

    throw new ApiRequestError(
      ERROR_MESSAGES[code] ??
        payload?.error.message ??
        `The decoder returned HTTP ${response.status}.`,
      code,
      response.status,
    );
  }

  return (await response.json()) as DecodedInvoice;
}
