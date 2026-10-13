export function normalizeInvoice(value: string): string {
  return value.trim().replace(/^lightning:/i, "").trim();
}

export async function readQrImage(file: File): Promise<string> {
  if (!/\.(png|jpe?g|webp)$/i.test(file.name)) {
    throw new Error("Choose a PNG, JPEG, or WebP image.");
  }
  if (file.size > 10 * 1024 * 1024) {
    throw new Error("Choose an image smaller than 10 MB.");
  }

  const url = URL.createObjectURL(file);
  try {
    const image = new Image();
    image.src = url;
    try {
      await image.decode();
    } catch {
      throw new Error("This image could not be opened. Choose a valid PNG, JPEG, or WebP file.");
    }
    if (image.naturalWidth * image.naturalHeight > 16_000_000) {
      throw new Error("Choose an image with fewer than 16 million pixels.");
    }
    const { BrowserQRCodeReader } = await import("@zxing/browser");
    let value: string;
    try {
      value = (await new BrowserQRCodeReader().decodeFromImageElement(image)).getText();
    } catch {
      throw new Error("No readable QR code found. Try a sharper image with the whole code visible.");
    }
    const invoice = normalizeInvoice(value);
    if (!/^ln(?:bcrt|tbs|bc|tb)[0-9]/i.test(invoice)) {
      throw new Error("This QR code does not contain a BOLT11 Lightning invoice.");
    }
    if (invoice.length > 8192) throw new Error("The invoice in this QR code is too long.");
    return invoice;
  } finally {
    URL.revokeObjectURL(url);
  }
}
