"use client";

import { useState, useEffect } from "react";
import { Button } from "./button";
import { GitHubStars } from "./github-stars";

export function Header() {
  const [pastHero, setPastHero] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      // Only show styled header when past the hero section
      setPastHero(window.scrollY > window.innerHeight - 100);
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-[background-color,border-color,backdrop-filter] duration-300 ${
        pastHero
          ? "border-b border-white/10 bg-[#050510]/80 backdrop-blur-md"
          : "bg-transparent"
      }`}
    >
      <nav className="w-full px-6 py-4 flex justify-end">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="sm"
            href="https://docs.axel.build"
          >
            Docs
          </Button>
          <GitHubStars scrolled={false} />
        </div>
      </nav>
    </header>
  );
}
