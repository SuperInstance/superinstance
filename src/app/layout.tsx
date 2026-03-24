import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "SuperInstance Ranch - Self-Evolving AI Ecosystem",
  description: "Don't rent an AI brain. Breed a Ranch that evolves forever. Self-evolving AI agents for Jetson Orin Nano 8GB with Night School breeding, Open Genomics, and CRDT memory.",
  keywords: ["SuperInstance", "AI Ranch", "Jetson Orin Nano", "TensorRT-LLM", "Self-Evolving AI", "Local AI", "LoRA", "Border Collie", "Open Genomics", "breed.md"],
  authors: [{ name: "SuperInstance Team" }],
  icons: {
    icon: "/logo.svg",
  },
  openGraph: {
    title: "SuperInstance Ranch - Self-Evolving AI Ecosystem",
    description: "Don't rent an AI brain. Breed a Ranch that evolves forever.",
    url: "https://superinstance.ai",
    siteName: "SuperInstance",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "SuperInstance Ranch",
    description: "Don't rent an AI brain. Breed a Ranch that evolves forever.",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased bg-background text-foreground`}
      >
        {children}
        <Toaster />
      </body>
    </html>
  );
}
