import type { ReactNode } from "react";
import { CommandBlock } from "@/components/command-block";

type Guide = { slug: string; title: string; description: string; sections: { id: string; title: string; content: ReactNode }[] };

export const guides: Guide[] = [
  {
    slug: "web", title: "Use the web decoder", description: "Paste an invoice, read the summary, and export the result.",
    sections: [
      { id: "input", title: "Start with a payment request", content: <><p>Paste a complete BOLT11 invoice into the decoder. If you copied it from Polar’s JSON, use only the <code>payment_request</code> value. You can also choose a sample or import a QR image.</p><p>A mainnet invoice starts with <code>lnbc</code>. Testnet uses <code>lntb</code>, signet uses <code>lntbs</code>, and regtest uses <code>lnbcrt</code>. A payment hash, Lightning address, or entire JSON response is not a BOLT11 invoice.</p></> },
      { id: "decode", title: "Decode and inspect", content: <><p>Optionally choose an expected network, then select Decode invoice. The summary shows the requested amount, description, and detected network. The validity card shows the signature, creation time, and expiry.</p><p>The validation report follows the summary. Payment identifiers have copy buttons, and Advanced details contains fallback addresses, description hash, CLTV delta, and route hints.</p></> },
      { id: "export", title: "Export or start again", content: <><p>Select Export JSON to download all decoded fields and the validation report. Select Decode another invoice to return to the input.</p><p>The built-in samples are expired public examples. Their expiry warnings are expected. Decoding does not make a payment.</p></> },
      { id: "connection", title: "How the web app connects", content: <p>QR images are read in your browser. Only the extracted invoice text is sent to the configured Rust API when you decode. The web app needs an available API; use the terminal client for fully offline decoding.</p> },
    ],
  },
  {
    slug: "setup", title: "Run locally", description: "Start the Rust API and Next.js frontend on your computer.",
    sections: [
      { id: "requirements", title: "Prepare your environment", content: <p>You need the project checkout, Rust 1.91 or newer with Cargo, and Node.js compatible with Next.js 16 plus npm. Run the commands from your project root unless a step says otherwise. Initial dependency installation needs internet access.</p> },
      { id: "backend", title: "Start the API", content: <><p>Create the local configuration if it does not exist, then start the backend. These commands use macOS / Linux shell syntax.</p><CommandBlock command={'cp -n .env.example .env\ncargo run -p lightning-api'} /><p>The default API address is <code>http://localhost:3001</code>. Leave this terminal running.</p></> },
      { id: "frontend", title: "Start the frontend", content: <><p>Open a second terminal at the project root:</p><CommandBlock command={'cd frontend\ncp -n .env.example .env.local\nnpm install\nnpm run dev -- --port 3000'} /><p>In <code>frontend/.env.local</code>, set <code>NEXT_PUBLIC_API_BASE_URL=http://localhost:3001</code>. Open <code>http://localhost:3000</code>. Restart the frontend after changing this variable; deployed builds need to be rebuilt.</p></> },
      { id: "verify", title: "Check the connection", content: <><CommandBlock command="curl http://localhost:3001/health" /><p>A healthy API returns <code>{'{"status":"ok"}'}</code>. Stop either development server with Ctrl+C in its terminal.</p></> },
    ],
  },
  {
    slug: "docker", title: "Run the API with Docker", description: "Build the backend image, map its port, and connect the frontend.",
    sections: [
      { id: "build", title: "Build the image", content: <><p>Start Docker, then run this from the project root. The first build downloads the base images and Rust dependencies.</p><CommandBlock command="docker build -t lightening-decoder-api ." /><p>The multi-stage build compiles Rust in the builder image and copies the API executable into a smaller runtime image.</p></> },
      { id: "run", title: "Start the container", content: <><CommandBlock command={'docker run --rm -p 3001:10000 \\\n  -e PORT=10000 \\\n  -e PUBLIC_BASE_URL=http://localhost:3001 \\\n  -e ALLOWED_ORIGINS=http://localhost:3000 \\\n  lightening-decoder-api'} /><p>This maps port 3001 on your computer to port 10000 in the container. Use this instead of running the Cargo backend on the same port. Ctrl+C stops this foreground container.</p></> },
      { id: "connect", title: "Connect your frontend", content: <><p>Set <code>NEXT_PUBLIC_API_BASE_URL=http://localhost:3001</code> in <code>frontend/.env.local</code> and restart the frontend. Docker runs the API only; start the frontend separately using the local setup guide.</p><CommandBlock command="curl http://localhost:3001/health" /></> },
      { id: "timeouts", title: "Inspect build timeouts", content: <><CommandBlock command="docker build --progress=plain -t lightening-decoder-api ." /><p><code>--progress=plain</code> makes logs easier to read; it does not speed up the build. Registry download timeouts indicate connectivity problems. Check your network, VPN, or Docker proxy. Cached dependencies can be reused on the next build.</p><p>Cargo resolves the workspace, so download errors can mention terminal dependencies even when building the API.</p></> },
    ],
  },
  {
    slug: "qr", title: "QR images and Polar", description: "Get an invoice, turn it into a QR, and import it into either client.",
    sections: [
      { id: "polar", title: "Get an invoice from Polar", content: <><p>Open the terminal for a running LND node in Polar. List that node’s invoices:</p><CommandBlock command="lncli --network=regtest listinvoices" /><p>Copy a complete <code>payment_request</code> value. To create a fresh demo invoice for 1,000 sats:</p><CommandBlock command={'lncli --network=regtest addinvoice --amt 1000 --memo "Decoder demo"'} /><p>These commands run inside the Polar node terminal, not the decoder’s terminal.</p></> },
      { id: "generate", title: "Generate a QR image", content: <><p>Back in your project’s local terminal, replace the placeholder with that invoice. Keep the quotes.</p><CommandBlock command="cargo run -p lightning-cli --example sample_qr -- 'PASTE_COMPLETE_INVOICE_HERE' my-invoice.png" /><p>This creates a PNG containing the invoice, with a <code>lightning:</code> prefix. Choose a new output filename if the file already exists. Encoding a QR does not create a new invoice or change its expiry.</p></> },
      { id: "import", title: "Import the QR", content: <><p>In the web app, choose or drop an image, review the extracted text, and select Decode invoice. In Ratatui, press F3 and enter the local image path.</p><p>Both clients support PNG, JPEG, and WebP up to 10 MB. The browser allows up to 16 million pixels; the terminal limits dimensions to 4096 × 4096. Use one QR per image.</p><p>A wallet’s BOLT11 invoice QR works too. Lightning address, LNURL, and BOLT12 offer QR codes are not currently supported as invoice input.</p></> },
      { id: "samples", title: "Use the bundled samples", content: <><p>Find the coffee and donation images in <code>frontend/public/samples/</code>. Regenerate them with:</p><CommandBlock command="cargo run -p lightning-cli --example sample_qr" /><p>These public sample invoices are expired and intended for inspection.</p></> },
    ],
  },
  {
    slug: "validation", title: "Understand validation", description: "Read the individual checks and understand what they establish.",
    sections: [
      { id: "checks", title: "What the report checks", content: <ul><li><strong>Format:</strong> Can the BOLT11 encoding be parsed?</li><li><strong>Signature:</strong> Does the invoice signature verify?</li><li><strong>Requirements:</strong> Does the invoice meet the parser’s required-field and semantic checks?</li><li><strong>Expiry:</strong> Has the invoice expired at the time of inspection?</li><li><strong>Network:</strong> Which network is encoded, and does it match your optional expectation?</li><li><strong>Amount:</strong> Is an amount specified, or does the payer choose it?</li></ul> },
      { id: "statuses", title: "Read the status labels", content: <ul><li><strong>Passed:</strong> The check succeeded.</li><li><strong>Warning:</strong> Review a condition such as expiry or a network mismatch.</li><li><strong>Failed:</strong> A validation check failed.</li><li><strong>Info:</strong> Context, such as an unspecified amount or detected network.</li><li><strong>Not checked:</strong> A preceding failure prevented this check.</li></ul> },
      { id: "limits", title: "Understand the limits", content: <><p>Expired invoices and network mismatches remain inspectable. Expiry is a snapshot at the report’s checked time and depends on the clock of the machine doing the validation.</p><p>A valid signature does not establish a recipient’s real-world identity. Decoding cannot establish whether an invoice has already been paid, whether a route is available, or whether your wallet supports every invoice feature. The report does not make an overall “safe to pay” claim.</p></> },
    ],
  },
  {
    slug: "api", title: "API and remote mode", description: "Call the HTTP endpoints or connect the terminal client to an API.",
    sections: [
      { id: "health", title: "Check API health", content: <><p>Start your local or Docker backend first.</p><CommandBlock command="curl http://localhost:3001/health" /><p>For an interactive request interface, open <code>http://localhost:3001/swagger-ui/</code>. The schema is at <code>/api-docs/openapi.json</code>.</p></> },
      { id: "decode", title: "Decode through HTTP", content: <><p>Replace the placeholder with the complete plain BOLT11 string. The optional network query accepts mainnet, testnet, signet, or regtest.</p><CommandBlock command={'curl "http://localhost:3001/api/v1/invoices/decode/PASTE_COMPLETE_INVOICE_HERE?expected_network=regtest"'} /><p>The invoice is a URL path segment; URL-encode input when constructing requests programmatically. Responses use millisatoshis for amounts and Unix seconds for timestamps, and include a validation object.</p></> },
      { id: "remote", title: "Use the terminal with the API", content: <><p>After building the CLI, launch remote mode:</p><CommandBlock command="./target/release/bolt11-decoder --api-url http://localhost:3001" /><p>Or decode directly:</p><CommandBlock command="./target/release/bolt11-decoder --api-url http://localhost:3001 decode 'PASTE_COMPLETE_INVOICE_HERE'" /><p>You can also configure <code>LIGHTNING_API_URL</code>. Remote mode requires the API to be reachable. The CLI validates locally first and attaches its local validation report to the remote result; QR images are always read locally.</p></> },
      { id: "errors", title: "Read API errors", content: <ul><li><strong>400:</strong> Invalid input or unsupported expected network.</li><li><strong>404:</strong> Unknown route; check the base URL and service on that port.</li><li><strong>414:</strong> Invoice exceeds the input limit.</li><li><strong>422:</strong> A decoded value cannot be represented safely.</li></ul> },
    ],
  },
  {
    slug: "troubleshooting", title: "Troubleshooting", description: "Resolve connection errors, QR issues, and expected validation warnings.",
    sections: [
      { id: "connection", title: "The decoder returns 404 or cannot connect", content: <><p>Check the API health endpoint:</p><CommandBlock command="curl -i http://localhost:3001/health" /><p>The Rust API should respond with JSON. A Next.js HTML page suggests that the frontend is running on the API port. Use port 3000 for the frontend and 3001 for the API.</p><p>Check <code>NEXT_PUBLIC_API_BASE_URL</code> in the frontend environment and restart or rebuild the frontend after changes. If the browser reports CORS errors, ensure <code>ALLOWED_ORIGINS</code> includes the frontend’s exact origin.</p></> },
      { id: "ports", title: "A port is already in use", content: <><p>On macOS or Linux with lsof installed, inspect the listener:</p><CommandBlock command="lsof -nP -iTCP:3000 -sTCP:LISTEN" /><p>For Docker containers publishing that port:</p><CommandBlock command="docker ps --filter publish=3000" /><p>Stop the identified service in its terminal or Docker Desktop, or choose another port. If the frontend origin changes, update the API’s allowed origins too.</p></> },
      { id: "qr", title: "A QR image will not import", content: <p>Use a clear image with the whole QR and its surrounding margin visible. Confirm it contains a BOLT11 invoice, uses a supported image format, and fits the size limits. You can paste the invoice text directly to distinguish an image-reading problem from an invoice-validation failure.</p> },
      { id: "warnings", title: "Samples are expired or the network does not match", content: <p>The bundled samples are intentionally expired. Polar invoices use regtest, so expecting mainnet will produce a mismatch warning. Select Detect only or the correct expected network. Neither warning prevents inspection.</p> },
      { id: "offline", title: "Offline mode tries to contact an API", content: <><p>Remove the environment setting in the terminal session and launch without an API flag:</p><CommandBlock command={'unset LIGHTNING_API_URL\n./target/release/bolt11-decoder'} /><p>If Cargo’s offline build reports missing dependencies, build once while connected. Running an already-built binary does not require Cargo or dependency downloads.</p></> },
    ],
  },
];
