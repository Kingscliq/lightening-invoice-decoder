import type { DecodedInvoice } from "@/types/invoice";

export async function decodeInvoice(invoice: string): Promise<DecodedInvoice> {
  void invoice;

  // TODO(lesson 9): Read NEXT_PUBLIC_API_BASE_URL.
  // TODO(lesson 9): URL-encode the invoice and call the Axum GET endpoint.
  // TODO(lesson 9): Parse successful JSON and present structured API errors.
  throw new Error(
    "Frontend API integration is not implemented yet; see frontend/lib/api.ts",
  );
}
