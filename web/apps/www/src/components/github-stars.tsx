"use client";

import { useEffect, useState } from "react";
import { Star } from "lucide-react";
import { Button } from "./button";

export function GitHubStars({ scrolled }: { scrolled?: boolean }) {
  const [stars, setStars] = useState<number | null>(null);

  useEffect(() => {
    fetch("https://api.github.com/repos/lgalabru/axel")
      .then((res) => res.json())
      .then((data) => setStars(data.stargazers_count ?? null))
      .catch(() => setStars(null));
  }, []);

  return (
    <Button
      variant="yellow"
      size="sm"
      href="https://github.com/lgalabru/axel"
      target="_blank"
      rel="noopener noreferrer"
      className="!gap-1"
    >
      <Star className="h-3 w-3 fill-current" aria-hidden="true" />
      {stars !== null ? stars.toLocaleString() : "—"}
    </Button>
  );
}
