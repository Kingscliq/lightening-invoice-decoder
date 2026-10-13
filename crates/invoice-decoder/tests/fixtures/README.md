# Invoice fixtures

Add stable BOLT11 samples here as the decoder test matrix grows. Keep each fixture's network, expected fields, and whether its signature is intentionally corrupted documented in the test that uses it.


`invalid-signature.txt` and `missing-secret.txt` are public BOLT11 examples from
https://github.com/lightning/bolts/blob/master/11-payment-encoding.md#examples.
The former has an unrecoverable signature; the latter has a valid signature but
lacks the required payment-secret field. Validation tests verify that these
fail at different stages and never claim downstream checks passed.
