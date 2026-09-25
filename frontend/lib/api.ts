import type { ApiErrorResponse, DecodedInvoice } from "@/types/invoice";

const API_BASE_URL = (
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3001"
).replace(/\/$/, "");

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
    throw new ApiRequestError("Enter a BOLT11 invoice.", "EMPTY_INVOICE", 400);
  }

  const response = await fetch(
    `${API_BASE_URL}/api/v1/invoices/decode/${encodeURIComponent(normalizedInvoice)}`,
    {
      headers: { Accept: "application/json" },
      cache: "no-store",
    },
  );

  if (!response.ok) {
    let payload: ApiErrorResponse | null = null;

    try {
      payload = (await response.json()) as ApiErrorResponse;
    } catch {
      // The server or an upstream proxy may return a non-JSON error page.
    }

    throw new ApiRequestError(
      payload?.error.message ?? `The API returned HTTP ${response.status}.`,
      payload?.error.code ?? "API_ERROR",
      response.status,
    );
  }

  return (await response.json()) as DecodedInvoice;
}
