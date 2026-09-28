import type { Metadata } from "next";
import { Instrument_Sans } from "next/font/google";
import type { ReactNode } from "react";
import "./globals.css";

const instrumentSans = Instrument_Sans({
  subsets: ["latin"],
  variable: "--font-instrument-sans",
});

export const metadata: Metadata = {
  title: "Lightening Decoder",
  description: "Decode and validate BOLT11 Lightning invoices.",
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="en">
      <body className={instrumentSans.variable}>{children}</body>
    </html>
  );
}
