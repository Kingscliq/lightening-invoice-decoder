export type RouteHintHop = {
  source_node_id: string;
  short_channel_id: number;
  base_fee_msat: number;
  proportional_fee_millionths: number;
  cltv_expiry_delta: number;
  htlc_minimum_msat: number | null;
  htlc_maximum_msat: number | null;
};

export type RouteHint = {
  hops: RouteHintHop[];
};

export type DecodedInvoice = {
  network: string;
  amount_msat: number | null;
  description: string | null;
  description_hash: string | null;
  payment_hash: string;
  payee_public_key: string;
  created_at_unix: number;
  expires_at_unix: number;
  expiry_seconds: number;
  expired: boolean;
  signature_valid: boolean;
  min_final_cltv_expiry_delta: number;
  fallback_addresses: string[];
  route_hints: RouteHint[];
};

export type ApiErrorResponse = {
  error: {
    code: string;
    message: string;
  };
};
