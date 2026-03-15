"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import {
  Bot,
  Shield,
  DollarSign,
  Inbox,
  Sparkles,
  Users,
  ArrowRight,
  Terminal,
  Check,
  FileEdit,
  Play,
  FileText,
} from "lucide-react";

function AppleLogo({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 15.7715 20.4004"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path d="M11.3965 4.66797C9.94141 4.66797 8.75 5.55664 7.98828 5.55664C7.17773 5.55664 6.12305 4.72656 4.85352 4.72656C2.44141 4.72656 0 6.71875 0 10.4688C0 12.8125 0.898438 15.2832 2.02148 16.875C2.97852 18.2227 3.81836 19.3262 5.0293 19.3262C6.2207 19.3262 6.74805 18.5352 8.23242 18.5352C9.73633 18.5352 10.0781 19.3066 11.3965 19.3066C12.7051 19.3066 13.5742 18.1055 14.4043 16.9238C15.3223 15.5664 15.7129 14.248 15.7227 14.1797C15.6445 14.1602 13.1445 13.1348 13.1445 10.2734C13.1445 7.79297 15.1074 6.67969 15.2246 6.5918C13.9258 4.72656 11.9434 4.66797 11.3965 4.66797ZM10.7129 3.08594C11.3086 2.36328 11.7285 1.37695 11.7285 0.380859C11.7285 0.244141 11.7188 0.107422 11.6992 0C10.7227 0.0390625 9.55078 0.644531 8.85742 1.46484C8.30078 2.08984 7.79297 3.08594 7.79297 4.08203C7.79297 4.23828 7.82227 4.38477 7.83203 4.43359C7.89062 4.44336 7.98828 4.46289 8.0957 4.46289C8.96484 4.46289 10.0586 3.87695 10.7129 3.08594Z" />
    </svg>
  );
}

function Starfield({ warp }: { warp: boolean }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const starsRef = useRef<
    { x: number; y: number; z: number; purple: boolean }[]
  >([]);
  const speedRef = useRef(0.2);
  const animationRef = useRef<number>(0);
  const prefersReducedMotion = useRef(false);

  const initStars = useCallback((width: number, height: number) => {
    const count = Math.min(500, Math.floor((width * height) / 3500));
    const stars: { x: number; y: number; z: number; purple: boolean }[] = [];
    for (let i = 0; i < count; i++) {
      stars.push({
        x: (Math.random() - 0.5) * width * 2,
        y: (Math.random() - 0.5) * height * 2,
        z: Math.random() * 1000,
        purple: Math.random() < 0.3,
      });
    }
    starsRef.current = stars;
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const mql = window.matchMedia("(prefers-reduced-motion: reduce)");
    prefersReducedMotion.current = mql.matches;
    const handleChange = (e: MediaQueryListEvent) => {
      prefersReducedMotion.current = e.matches;
    };
    mql.addEventListener("change", handleChange);

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      if (starsRef.current.length === 0) {
        initStars(canvas.width, canvas.height);
      }
    };
    resize();
    window.addEventListener("resize", resize);

    const draw = () => {
      if (!ctx || !canvas) return;

      const { width, height } = canvas;
      const cx = width / 2;
      const cy = height / 2;

      // Smoothly interpolate speed with ease-in curve
      const targetSpeed = prefersReducedMotion.current ? 0 : warp ? 14 : 0.2;
      const easeFactor = warp ? 0.025 : 0.015;
      speedRef.current += (targetSpeed - speedRef.current) * easeFactor;

      ctx.fillStyle = "#050510";
      ctx.fillRect(0, 0, width, height);

      // Subtle center glow during warp
      const warpIntensity = Math.min(1, speedRef.current / 10);
      if (warpIntensity > 0.05) {
        const gradient = ctx.createRadialGradient(
          cx,
          cy,
          0,
          cx,
          cy,
          Math.max(width, height) * 0.5,
        );
        gradient.addColorStop(0, `rgba(124, 58, 237, ${warpIntensity * 0.06})`);
        gradient.addColorStop(
          0.4,
          `rgba(99, 102, 241, ${warpIntensity * 0.03})`,
        );
        gradient.addColorStop(1, "transparent");
        ctx.fillStyle = gradient;
        ctx.fillRect(0, 0, width, height);
      }

      const stars = starsRef.current;
      for (let i = 0; i < stars.length; i++) {
        const star = stars[i];
        star.z -= speedRef.current;

        if (star.z <= 0) {
          star.x = (Math.random() - 0.5) * width * 2;
          star.y = (Math.random() - 0.5) * height * 2;
          star.z = 1000;
          star.purple = Math.random() < 0.3;
        }

        const sx = (star.x / star.z) * 300 + cx;
        const sy = (star.y / star.z) * 300 + cy;

        if (sx < 0 || sx > width || sy < 0 || sy > height) {
          star.x = (Math.random() - 0.5) * width * 2;
          star.y = (Math.random() - 0.5) * height * 2;
          star.z = 1000;
          star.purple = Math.random() < 0.3;
          continue;
        }

        const depth = 1 - star.z / 1000;
        const size = Math.max(0.3, depth * (1.8 + warpIntensity * 0.5));
        const opacity = Math.max(0.05, depth * (0.6 + warpIntensity * 0.3));

        // Streaks that grow with speed
        if (speedRef.current > 0.4) {
          const trailLength = speedRef.current * 3;
          const prevZ = star.z + trailLength;
          const prevSx = (star.x / prevZ) * 300 + cx;
          const prevSy = (star.y / prevZ) * 300 + cy;
          const trailOpacity =
            opacity * 0.5 * Math.min(1, speedRef.current / 5);

          const grad = ctx.createLinearGradient(prevSx, prevSy, sx, sy);
          if (star.purple) {
            grad.addColorStop(0, `rgba(167, 139, 250, 0)`);
            grad.addColorStop(1, `rgba(167, 139, 250, ${trailOpacity})`);
          } else {
            grad.addColorStop(0, `rgba(180, 180, 255, 0)`);
            grad.addColorStop(1, `rgba(220, 220, 255, ${trailOpacity})`);
          }

          ctx.beginPath();
          ctx.moveTo(prevSx, prevSy);
          ctx.lineTo(sx, sy);
          ctx.strokeStyle = grad;
          ctx.lineWidth = size * 0.6;
          ctx.stroke();
        }

        ctx.beginPath();
        ctx.arc(sx, sy, size, 0, Math.PI * 2);
        ctx.fillStyle = star.purple
          ? `rgba(167, 139, 250, ${opacity})`
          : `rgba(255, 255, 255, ${opacity})`;
        ctx.fill();
      }

      animationRef.current = requestAnimationFrame(draw);
    };

    animationRef.current = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animationRef.current);
      window.removeEventListener("resize", resize);
      mql.removeEventListener("change", handleChange);
    };
  }, [warp, initStars]);

  return (
    <canvas ref={canvasRef} className="fixed inset-0 z-0" aria-hidden="true" />
  );
}

function BentoCard({
  icon: Icon,
  title,
  description,
  className = "",
  accent = false,
}: {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description: string;
  className?: string;
  accent?: boolean;
}) {
  return (
    <div
      className={`group relative overflow-hidden rounded-2xl p-6 transition-all duration-300 ${
        accent
          ? "bg-gradient-to-br from-purple-600 to-violet-700 text-white"
          : "bg-gray-50 hover:bg-gray-100/80"
      } ${className}`}
    >
      <div
        className={`mb-5 inline-flex h-10 w-10 items-center justify-center rounded-xl ${
          accent
            ? "bg-white/20 text-white"
            : "bg-white text-purple-600 shadow-sm"
        }`}
        aria-hidden="true"
      >
        <Icon className="h-5 w-5" />
      </div>
      <h3
        className={`mb-2 text-lg font-semibold tracking-tight ${
          accent ? "text-white" : "text-gray-900"
        }`}
      >
        {title}
      </h3>
      <p
        className={`text-sm leading-relaxed ${
          accent ? "text-white/80" : "text-gray-500"
        }`}
      >
        {description}
      </p>
    </div>
  );
}

function InboxCard({
  type,
  title,
  description,
  time,
  urgent,
  resolved,
}: {
  type: "edit" | "command" | "read";
  title: string;
  description: string;
  time: string;
  urgent?: boolean;
  resolved?: boolean;
}) {
  const typeIcons = {
    edit: <FileEdit className="h-4 w-4" />,
    command: <Play className="h-4 w-4" />,
    read: <FileText className="h-4 w-4" />,
  };

  const typeColors = {
    edit: "text-orange-600 bg-orange-100",
    command: "text-blue-600 bg-blue-100",
    read: "text-gray-600 bg-gray-100",
  };

  return (
    <div
      className={`bg-white rounded-2xl border shadow-lg p-4 transition-all ${
        resolved
          ? "border-gray-200 opacity-60"
          : urgent
            ? "border-orange-300 shadow-orange-100"
            : "border-gray-200"
      }`}
    >
      <div className="flex items-start gap-3">
        <div
          className={`flex-shrink-0 h-8 w-8 rounded-lg flex items-center justify-center ${typeColors[type]}`}
        >
          {typeIcons[type]}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <p className="text-sm font-semibold text-gray-900 truncate">
              {title}
            </p>
            {urgent && !resolved && (
              <span className="flex-shrink-0 h-2 w-2 rounded-full bg-orange-500 animate-pulse" />
            )}
          </div>
          <p className="text-xs text-gray-500 mb-2">{description}</p>
          <div className="flex items-center justify-between">
            <span className="text-xs text-gray-400">{time}</span>
            {!resolved ? (
              <div className="flex gap-2">
                <button className="text-xs px-3 py-1 rounded-full bg-gray-100 text-gray-600 hover:bg-gray-200 transition-colors">
                  Deny
                </button>
                <button className="text-xs px-3 py-1 rounded-full bg-purple-600 text-white hover:bg-purple-500 transition-colors">
                  Approve
                </button>
              </div>
            ) : (
              <span className="text-xs text-green-600 flex items-center gap-1">
                <Check className="h-3 w-3" />
                Auto-resolved
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function GhosttyLogo({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M12 2a7 7 0 0 0-7 7v8a3 3 0 0 0 3 3h1v-2a2 2 0 1 1 4 0v2h2v-2a2 2 0 1 1 4 0v2h1a3 3 0 0 0 3-3V9a7 7 0 0 0-7-7z" />
      <circle cx="9" cy="10" r="1.5" fill="currentColor" stroke="none" />
      <circle cx="15" cy="10" r="1.5" fill="currentColor" stroke="none" />
    </svg>
  );
}

function SkillDistributionGraphic() {
  return (
    <div className="relative h-52">
      <style>{`
        @keyframes electronMove {
          0% { stroke-dashoffset: 200; }
          100% { stroke-dashoffset: -200; }
        }
        .electron-claude { animation: electronMove 1.5s ease-in-out infinite; }
        .electron-codex { animation: electronMove 1.7s ease-in-out infinite 0.2s; }
        .electron-antigravity { animation: electronMove 1.6s ease-in-out infinite 0.6s; }
        .electron-opencode { animation: electronMove 1.8s ease-in-out infinite 0.4s; }
      `}</style>

      <div className="absolute inset-0 grid grid-cols-[1fr_1.5fr_1fr] items-center">
        {/* Left: Skill files */}
        <div className="flex flex-col gap-4">
          <div className="flex items-center gap-2 rounded-lg border border-purple-200 bg-purple-50 px-3 py-2.5 shadow-sm">
            <FileText className="h-4 w-4 text-purple-500" />
            <span className="text-xs font-mono text-gray-700">
              rust-dev.md
            </span>
          </div>
          <div className="flex items-center gap-2 rounded-lg border border-purple-200 bg-purple-50 px-3 py-2.5 shadow-sm">
            <FileText className="h-4 w-4 text-purple-500" />
            <span className="text-xs font-mono text-gray-700">
              web-dev.md
            </span>
          </div>
        </div>

        {/* Middle: Connection lines + AXEL icon */}
        <div className="relative h-full">
          <svg
            className="absolute inset-0 h-full w-full"
            viewBox="0 0 120 180"
            preserveAspectRatio="none"
          >
            <defs>
              <filter id="glow-claude" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="3" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              <filter id="glow-codex" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="3" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              <filter id="glow-antigravity" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="3" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
              <filter id="glow-opencode" x="-50%" y="-50%" width="200%" height="200%">
                <feGaussianBlur stdDeviation="3" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
            </defs>

            {/* Base paths (dim) */}
            <path d="M 0 65 Q 60 90, 120 25" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 65 Q 60 90, 120 70" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 65 Q 60 90, 120 115" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 65 Q 60 90, 120 160" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 115 Q 60 90, 120 25" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 115 Q 60 90, 120 70" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 115 Q 60 90, 120 115" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />
            <path d="M 0 115 Q 60 90, 120 160" fill="none" stroke="#e5e7eb" strokeWidth="1" opacity="0.5" />

            {/* Claude electrons (tan/orange) */}
            <path d="M 0 65 Q 60 90, 120 25" fill="none" stroke="#D4A27F" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-claude)" className="electron-claude" />
            <path d="M 0 115 Q 60 90, 120 25" fill="none" stroke="#D4A27F" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-claude)" className="electron-claude" />

            {/* Codex electrons (green) */}
            <path d="M 0 65 Q 60 90, 120 70" fill="none" stroke="#10A37F" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-codex)" className="electron-codex" />
            <path d="M 0 115 Q 60 90, 120 70" fill="none" stroke="#10A37F" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-codex)" className="electron-codex" />

            {/* Antigravity electrons (blue) */}
            <path d="M 0 65 Q 60 90, 120 115" fill="none" stroke="#4285F4" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-antigravity)" className="electron-antigravity" />
            <path d="M 0 115 Q 60 90, 120 115" fill="none" stroke="#4285F4" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-antigravity)" className="electron-antigravity" />

            {/* OpenCode electrons (pink) */}
            <path d="M 0 65 Q 60 90, 120 160" fill="none" stroke="#EC4899" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-opencode)" className="electron-opencode" />
            <path d="M 0 115 Q 60 90, 120 160" fill="none" stroke="#EC4899" strokeWidth="3" strokeDasharray="15 185" strokeLinecap="round" filter="url(#glow-opencode)" className="electron-opencode" />
          </svg>

          {/* AXEL icon in center */}
          <div className="absolute inset-0 flex items-center justify-center z-10 translate-y-3 -translate-x-4">
            <div className="flex flex-col items-center">
              <div className="h-14 w-14 rounded-2xl bg-gradient-to-br from-purple-500 to-violet-600 flex items-center justify-center shadow-lg shadow-purple-500/30">
                <span className="text-white font-bold text-lg">A</span>
              </div>
              <span className="mt-2 text-xs font-semibold text-gray-600">axel</span>
            </div>
          </div>
        </div>

        {/* Right: LLM icons */}
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2.5 rounded-lg border border-gray-200 bg-white px-3 py-2 shadow-sm">
            <svg className="h-6 w-6" viewBox="0 0 24 24" fill="none">
              <path d="M17.304 4.794l-6.163 14.412h-2.845l6.163-14.412h2.845zm-7.758 0l-6.163 14.412h2.845l6.163-14.412h-2.845z" fill="#D4A27F" />
            </svg>
            <span className="text-xs font-medium text-gray-700">Claude</span>
          </div>
          <div className="flex items-center gap-2.5 rounded-lg border border-gray-200 bg-white px-3 py-2 shadow-sm">
            <svg className="h-6 w-6" viewBox="0 0 24 24" fill="none">
              <path d="M22.282 9.821a5.985 5.985 0 0 0-.516-4.91 6.046 6.046 0 0 0-6.51-2.9A6.065 6.065 0 0 0 4.981 4.18a5.985 5.985 0 0 0-3.998 2.9 6.046 6.046 0 0 0 .743 7.097 5.98 5.98 0 0 0 .51 4.911 6.051 6.051 0 0 0 6.515 2.9A5.985 5.985 0 0 0 13.26 24a6.056 6.056 0 0 0 5.772-4.206 5.99 5.99 0 0 0 3.997-2.9 6.056 6.056 0 0 0-.747-7.073zM13.26 22.43a4.476 4.476 0 0 1-2.876-1.04l.141-.081 4.779-2.758a.795.795 0 0 0 .392-.681v-6.737l2.02 1.168a.071.071 0 0 1 .038.052v5.583a4.504 4.504 0 0 1-4.494 4.494zM3.6 18.304a4.47 4.47 0 0 1-.535-3.014l.142.085 4.783 2.759a.771.771 0 0 0 .78 0l5.843-3.369v2.332a.08.08 0 0 1-.033.062L9.74 19.95a4.5 4.5 0 0 1-6.14-1.646zM2.34 7.896a4.485 4.485 0 0 1 2.366-1.973V11.6a.766.766 0 0 0 .388.676l5.815 3.355-2.02 1.168a.076.076 0 0 1-.071 0l-4.83-2.786A4.504 4.504 0 0 1 2.34 7.872zm16.597 3.855l-5.833-3.387L15.119 7.2a.076.076 0 0 1 .071 0l4.83 2.791a4.494 4.494 0 0 1-.676 8.105v-5.678a.79.79 0 0 0-.407-.667zm2.01-3.023l-.141-.085-4.774-2.782a.776.776 0 0 0-.785 0L9.409 9.23V6.897a.066.066 0 0 1 .028-.061l4.83-2.787a4.5 4.5 0 0 1 6.68 4.66zm-12.64 4.135l-2.02-1.164a.08.08 0 0 1-.038-.057V6.075a4.5 4.5 0 0 1 7.375-3.453l-.142.08-4.778 2.758a.795.795 0 0 0-.393.681zm1.097-2.365l2.602-1.5 2.607 1.5v2.999l-2.597 1.5-2.607-1.5z" fill="#10A37F" />
            </svg>
            <span className="text-xs font-medium text-gray-700">Codex</span>
          </div>
          <div className="flex items-center gap-2.5 rounded-lg border border-gray-200 bg-white px-3 py-2 shadow-sm">
            <svg className="h-6 w-6" viewBox="0 0 24 24" fill="none">
              <defs>
                <linearGradient id="antigravity-grad" x1="50%" y1="0%" x2="50%" y2="100%">
                  <stop offset="0%" stopColor="#FF6B4A" />
                  <stop offset="30%" stopColor="#FFD93D" />
                  <stop offset="50%" stopColor="#6BCB77" />
                  <stop offset="100%" stopColor="#4D96FF" />
                </linearGradient>
              </defs>
              <path d="M12 2C12 2 6 10 4 20C4.5 20.5 5.5 21 6 20C7 17 9 14 12 12C15 14 17 17 18 20C18.5 21 19.5 20.5 20 20C18 10 12 2 12 2Z" fill="url(#antigravity-grad)" />
            </svg>
            <span className="text-xs font-medium text-gray-700">Antigravity</span>
          </div>
          <div className="flex items-center gap-2.5 rounded-lg border border-gray-200 bg-white px-3 py-2 shadow-sm">
            <svg className="h-6 w-6" viewBox="0 0 24 24" fill="none">
              <rect width="24" height="24" rx="4" fill="#EC4899" fillOpacity="0.2" />
              <path d="M8 8l-4 4 4 4M16 8l4 4-4 4M14 6l-4 12" stroke="#EC4899" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
            <span className="text-xs font-medium text-gray-700">OpenCode</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// Animation frames for scroll-driven typewriter
// Each frame: [display string, cursor position, x index (-1 if no x)]
const ANIMATION_FRAMES: [string, number, number][] = [
  ["Accelerate", 10, -1],
  ["Accelerat", 9, -1],
  ["Accelera", 8, -1],
  ["Acceler", 7, -1],
  ["Accele", 6, -1],
  ["Accel", 5, -1],
  ["Acel", 4, -1],
  ["Ael", 3, -1],
  ["Al", 2, -1],
  ["Axl", 2, 1],
  ["Axel", 4, 1],
];

function HeroHeadline({ scrollProgress }: { scrollProgress: number }) {
  // Map scroll progress (0-1) to animation frame
  const frameIndex = Math.min(
    ANIMATION_FRAMES.length - 1,
    Math.floor(scrollProgress * ANIMATION_FRAMES.length),
  );

  const [displayText, cursorPos, xIdx] = ANIMATION_FRAMES[frameIndex];
  const chars = displayText.split("");

  // Show cursor when animating (not at start or end)
  const showCursor = frameIndex > 0 && frameIndex < ANIMATION_FRAMES.length - 1;

  return (
    <h1
      className="text-7xl font-bold tracking-tight text-white sm:text-8xl md:text-9xl leading-none"
      style={{ fontFamily: "'Space Grotesk', sans-serif" }}
    >
      {chars.map((ch, i) => (
        <span key={`${i}-${ch}-${frameIndex}`}>
          {i === cursorPos && showCursor && (
            <span className="inline-block w-[4px] h-[0.75em] bg-purple-400 align-baseline animate-[blink_1s_step-end_infinite] -mr-[4px] relative z-10" />
          )}
          <span
            style={
              i === xIdx
                ? {
                    textShadow:
                      "0 0 10px rgba(167, 139, 250, 0.8), 0 0 20px rgba(167, 139, 250, 0.6), 0 0 30px rgba(167, 139, 250, 0.4), 0 0 40px rgba(167, 139, 250, 0.2)",
                  }
                : undefined
            }
          >
            {ch}
          </span>
        </span>
      ))}
      {cursorPos === chars.length && showCursor && (
        <span className="inline-block w-[4px] h-[0.75em] bg-purple-400 ml-1 align-baseline animate-[blink_1s_step-end_infinite]" />
      )}
    </h1>
  );
}

function ScrollIndicator({ visible }: { visible: boolean }) {
  return (
    <div
      className="fixed left-1/2 z-20 flex flex-col items-center gap-2"
      style={{
        bottom: "140px",
        transform: `translateX(-50%) translateY(${visible ? 0 : 16}px)`,
        opacity: visible ? 1 : 0,
        pointerEvents: visible ? "auto" : "none",
        transition: "all 500ms",
      }}
    >
      <span className="text-xs font-mono tracking-[0.3em] text-white/40 uppercase">
        Scroll
      </span>
      <div className="w-px h-8 bg-gradient-to-b from-white/40 to-transparent relative overflow-hidden">
        <div className="absolute inset-0 w-full bg-gradient-to-b from-purple-400 to-transparent animate-[scrollPulse_2s_ease-in-out_infinite]" />
      </div>
    </div>
  );
}

export default function Home() {
  const [warp, setWarp] = useState(false);
  const [scrollProgress, setScrollProgress] = useState(0);
  const scrollAccumulator = useRef(0);
  const isLocked = useRef(true);
  const justUnlocked = useRef(false);

  // Scroll-jacking: capture wheel events to drive animation while locked
  useEffect(() => {
    const typewriterThreshold = 500; // Scroll needed for typewriter
    const vibrateThreshold = 400; // Additional scroll for vibration

    const handleWheel = (e: WheelEvent) => {
      // When locked, capture scroll to drive animation
      if (isLocked.current) {
        e.preventDefault();

        // First phase: typewriter animation
        if (scrollAccumulator.current < typewriterThreshold) {
          scrollAccumulator.current = Math.max(
            0,
            Math.min(typewriterThreshold, scrollAccumulator.current + e.deltaY),
          );
          const progress = scrollAccumulator.current / typewriterThreshold;
          setScrollProgress(progress);
        }
        // Second phase: vibration
        else {
          const vibrateScroll = scrollAccumulator.current - typewriterThreshold;
          const newVibrateScroll = Math.max(
            0,
            Math.min(vibrateThreshold, vibrateScroll + e.deltaY),
          );
          scrollAccumulator.current = typewriterThreshold + newVibrateScroll;
          setVibrateProgress(newVibrateScroll / vibrateThreshold);
        }
      }
      // When at top of page and scrolling up, re-lock and reverse (but not right after unlock)
      else if (window.scrollY <= 0 && e.deltaY < 0 && !justUnlocked.current) {
        e.preventDefault();
        isLocked.current = true;
        setShowButton(false);
        setVibrateProgress(0);
        scrollAccumulator.current = typewriterThreshold + e.deltaY;
        const progress = Math.max(
          0,
          scrollAccumulator.current / typewriterThreshold,
        );
        setScrollProgress(progress);
      }
    };

    // Handle touch devices
    let touchStartY = 0;
    const handleTouchStart = (e: TouchEvent) => {
      touchStartY = e.touches[0].clientY;
    };

    const handleTouchMove = (e: TouchEvent) => {
      const deltaY = touchStartY - e.touches[0].clientY;
      touchStartY = e.touches[0].clientY;

      if (isLocked.current) {
        e.preventDefault();

        if (scrollAccumulator.current < typewriterThreshold) {
          scrollAccumulator.current = Math.max(
            0,
            Math.min(typewriterThreshold, scrollAccumulator.current + deltaY),
          );
          setScrollProgress(scrollAccumulator.current / typewriterThreshold);
        } else {
          const vibrateScroll = scrollAccumulator.current - typewriterThreshold;
          const newVibrateScroll = Math.max(
            0,
            Math.min(vibrateThreshold, vibrateScroll + deltaY),
          );
          scrollAccumulator.current = typewriterThreshold + newVibrateScroll;
          setVibrateProgress(newVibrateScroll / vibrateThreshold);
        }
      } else if (window.scrollY <= 0 && deltaY < 0 && !justUnlocked.current) {
        e.preventDefault();
        isLocked.current = true;
        setShowButton(false);
        setVibrateProgress(0);
        scrollAccumulator.current = typewriterThreshold + deltaY;
        setScrollProgress(
          Math.max(0, scrollAccumulator.current / typewriterThreshold),
        );
      }
    };

    window.addEventListener("wheel", handleWheel, { passive: false });
    window.addEventListener("touchstart", handleTouchStart, { passive: true });
    window.addEventListener("touchmove", handleTouchMove, { passive: false });

    return () => {
      window.removeEventListener("wheel", handleWheel);
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchmove", handleTouchMove);
    };
  }, []);

  const [vibrateProgress, setVibrateProgress] = useState(0);
  const [vibrateOffset, setVibrateOffset] = useState(0);
  const [showButton, setShowButton] = useState(false);
  const vibrateRef = useRef<number | null>(null);

  const typewriterComplete = scrollProgress >= 1;
  const isVibrating = typewriterComplete && vibrateProgress < 1;
  const vibrationComplete = typewriterComplete && vibrateProgress >= 1;
  const yokeScale = vibrationComplete ? 0.55 : 1;
  const yokeRotation = vibrationComplete ? 32 : 0;

  // Vibration effect using requestAnimationFrame
  useEffect(() => {
    if (!isVibrating) {
      setVibrateOffset(0);
      if (vibrateRef.current) cancelAnimationFrame(vibrateRef.current);
      return;
    }

    let lastTime = 0;
    const animate = (time: number) => {
      if (time - lastTime > 30) {
        setVibrateOffset((Math.random() - 0.5) * 6);
        lastTime = time;
      }
      vibrateRef.current = requestAnimationFrame(animate);
    };

    vibrateRef.current = requestAnimationFrame(animate);

    return () => {
      if (vibrateRef.current) cancelAnimationFrame(vibrateRef.current);
    };
  }, [isVibrating]);

  // Show button and unlock after vibration complete
  useEffect(() => {
    if (vibrationComplete && !showButton) {
      setShowButton(true);
      isLocked.current = false;
      justUnlocked.current = true;
      // Reset justUnlocked after a short delay
      setTimeout(() => {
        justUnlocked.current = false;
      }, 100);
    }
  }, [vibrationComplete, showButton]);

  // Warp stars during typing animation and vibration
  useEffect(() => {
    setWarp(isVibrating || (scrollProgress > 0 && scrollProgress < 1));
  }, [scrollProgress, isVibrating]);

  return (
    <div className="relative">
      <Starfield warp={warp} />

      {/* Hero - Full viewport initial state */}
      <div className="relative z-10 min-h-screen flex flex-col overflow-hidden">
        {/* Hero Section */}
        <section className="relative flex-1 flex flex-col items-center justify-center px-6 pb-32">
          {/* Headline - centered, offset up by 300px */}
          <div className="text-center -mt-[300px]">
            <HeroHeadline scrollProgress={scrollProgress} />
          </div>

          {/* Yoke - starts with only 64px visible, moves up as user scrolls */}
          <div
            className="absolute bottom-0 left-1/2 flex flex-col items-center"
            style={{
              transform: `translateX(calc(-50% + ${vibrateOffset}px)) translateY(calc(100% - 114px - ${scrollProgress * 488}px))`,
              transition: vibrationComplete
                ? "transform 500ms ease-out"
                : "none",
            }}
          >
            <div className="relative">
              {/* Background image - appears after vibration, does not rotate */}
              <img
                src="/axel-background.png"
                alt=""
                className={`absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 transition-all duration-500 ${
                  vibrationComplete
                    ? "opacity-100 scale-100"
                    : "opacity-0 scale-75"
                }`}
                style={{ width: "200px", height: "auto", zIndex: -1 }}
              />
              {/* Yoke image - rotates */}
              <img
                src="/axel-yoke.png"
                alt="Axel interface"
                style={{
                  width: "500px",
                  height: "auto",
                  transform: `scale(${yokeScale}) rotate(${yokeRotation}deg)`,
                  transition: vibrationComplete
                    ? "transform 500ms ease-out"
                    : "none",
                }}
              />
            </div>
            {/* Tagline - appears after shrink */}
            <p
              className={`mt-8 text-lg text-white/60 transition-all duration-500 ${
                showButton
                  ? "opacity-100 translate-y-0"
                  : "opacity-0 translate-y-4"
              }`}
            >
              Accelerated Task Manager for Mac
            </p>
            {/* CTA - appears after shrink */}
            <a
              href="https://github.com/lgalabru/axel/releases/latest/download/Axel-latest-macos.dmg"
              className={`mt-4 group inline-flex items-center gap-3 rounded-full bg-purple-600 px-12 py-5 text-xl font-semibold text-white shadow-xl shadow-purple-500/25 transition-all duration-500 delay-100 hover:bg-purple-500 hover:shadow-purple-500/40 hover:scale-[1.02] active:scale-[0.98] ${
                showButton
                  ? "opacity-100 translate-y-0"
                  : "opacity-0 translate-y-4 pointer-events-none"
              }`}
            >
              <AppleLogo className="h-6 w-6" />
              Download for Mac
            </a>
          </div>
        </section>

        {/* Scroll indicator */}
        <ScrollIndicator visible={!showButton} />
      </div>

      {/* White Section with Dashed Lines */}
      <div className="relative z-10 bg-white">
        {/* Dashed purple lines on sides */}
        <div className="absolute inset-0 mx-auto max-w-5xl pointer-events-none">
          <div className="absolute left-0 top-0 bottom-0 w-px border-l border-dashed border-purple-300" />
          <div className="absolute right-0 top-0 bottom-0 w-px border-r border-dashed border-purple-300" />
        </div>

        {/* Section 1: Task Queue */}
        <section className="relative mx-auto max-w-5xl px-6 py-24">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full bg-purple-100 px-4 py-1.5 mb-6">
                <Bot className="h-4 w-4 text-purple-600" />
                <span className="text-sm font-medium text-purple-700">
                  Task Queue
                </span>
              </div>
              <h2 className="text-4xl font-bold tracking-tight text-gray-900 mb-4">
                Queue tasks,
                <br />
                <span className="text-purple-600">dispatch to agents</span>
              </h2>
              <p className="text-lg text-gray-500 mb-8">
                Add tasks to a queue. Pick which agent runs each one. Reorder
                priorities on the fly. Watch them execute in parallel.
              </p>
              <div className="space-y-4">
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-purple-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-purple-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      Works with Claude, Codex, OpenCode, Antigravity
                    </p>
                    <p className="text-sm text-gray-500">
                      Same queue, different agents. Pick the right tool for the job.
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-purple-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-purple-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      Drag to reorder, click to dispatch
                    </p>
                    <p className="text-sm text-gray-500">
                      Change priorities while tasks run. No restart needed.
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-purple-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-purple-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">Keyboard shortcuts for everything</p>
                    <p className="text-sm text-gray-500">
                      New pane, dispatch, reorder, kill—never touch the mouse
                    </p>
                  </div>
                </div>
              </div>
            </div>
            {/* Task Queue Visual */}
            <div className="relative lg:-mr-6">
              <img
                src="/tasks.png"
                alt="Axel task queue showing running tasks"
                className="w-full h-auto scale-110 origin-right"
              />
              {/* Right edge dotted line */}
              <div className="absolute top-0 right-0 bottom-0 w-px border-r border-dashed border-purple-300" />
            </div>
          </div>
        </section>

        {/* Section 2: Terminal */}
        <section className="relative mx-auto max-w-5xl px-6 py-24">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            {/* Skill Distribution Visual */}
            <div className="relative order-2 lg:order-1">
              <div className="relative bg-gray-50 rounded-2xl border border-gray-200 shadow-xl overflow-hidden p-6">
                <SkillDistributionGraphic />
              </div>
            </div>
            <div className="order-1 lg:order-2">
              <div className="inline-flex items-center gap-2 rounded-full bg-indigo-100 px-4 py-1.5 mb-6">
                <Sparkles className="h-4 w-4 text-indigo-600" />
                <span className="text-sm font-medium text-indigo-700">
                  Portable Skills
                </span>
              </div>
              <h2 className="text-4xl font-bold tracking-tight text-gray-900 mb-4">
                One skill file,
                <br />
                <span className="text-indigo-600">every agent</span>
              </h2>
              <p className="text-lg text-gray-500 mb-8">
                Store skills in <code className="text-sm bg-gray-100 px-1.5 py-0.5 rounded">~/.config/axel/skills</code>. Axel symlinks them to each agent&apos;s expected location when you launch.
              </p>
              <div className="space-y-4">
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-indigo-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-indigo-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      <code className="text-sm bg-gray-100 px-1 py-0.5 rounded">axel -w feat/auth</code>
                    </p>
                    <p className="text-sm text-gray-500">
                      Spawns a git worktree + tmux session for that branch
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-indigo-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-indigo-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">AXEL.md defines your layout</p>
                    <p className="text-sm text-gray-500">
                      Panes, skills, grid positions—all in one YAML frontmatter
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-indigo-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-indigo-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      tmux or iTerm2
                    </p>
                    <p className="text-sm text-gray-500">
                      Sessions persist. Close your terminal, reattach later.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Section 3: Inbox */}
        <section className="relative mx-auto max-w-5xl px-6 py-24">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div className="inline-flex items-center gap-2 rounded-full bg-orange-100 px-4 py-1.5 mb-6">
                <Inbox className="h-4 w-4 text-orange-600" />
                <span className="text-sm font-medium text-orange-700">
                  Inbox
                </span>
              </div>
              <h2 className="text-4xl font-bold tracking-tight text-gray-900 mb-4">
                Approve or deny
                <br />
                <span className="text-orange-600">from one inbox</span>
              </h2>
              <p className="text-lg text-gray-500 mb-8">
                Agents request permission to edit files, run commands, make API calls.
                You see them all in one place. Approve, deny, or set rules to auto-approve.
              </p>
              <div className="space-y-4">
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-orange-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-orange-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      Full context on each request
                    </p>
                    <p className="text-sm text-gray-500">
                      File path, diff preview, command args—before you approve
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-orange-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-orange-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      Auto-approve rules
                    </p>
                    <p className="text-sm text-gray-500">
                      Skip the inbox for read-only ops or small edits under N tokens
                    </p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <div className="flex-shrink-0 h-6 w-6 rounded-full bg-orange-100 flex items-center justify-center mt-0.5">
                    <ArrowRight className="h-3 w-3 text-orange-600" />
                  </div>
                  <div>
                    <p className="font-medium text-gray-900">
                      macOS notifications
                    </p>
                    <p className="text-sm text-gray-500">
                      Get pinged when an agent is blocked waiting for approval
                    </p>
                  </div>
                </div>
              </div>
            </div>
            {/* Inbox Visual */}
            <div className="relative">
              <div className="absolute -inset-4 bg-gradient-to-r from-orange-100 to-amber-100 rounded-3xl blur-2xl opacity-60" />
              <div className="relative space-y-3">
                <InboxCard
                  type="edit"
                  title="Edit src/auth/middleware.ts"
                  description="Claude wants to modify the auth middleware to add JWT validation"
                  time="Just now"
                  urgent
                />
                <InboxCard
                  type="command"
                  title="Run: pnpm test"
                  description="Execute test suite in /packages/api"
                  time="2m ago"
                />
                <InboxCard
                  type="read"
                  title="Read package.json"
                  description="Access project dependencies"
                  time="5m ago"
                  resolved
                />
              </div>
            </div>
          </div>
        </section>

        {/* Bento Features Section */}
        <section className="relative mx-auto max-w-5xl px-6 py-24 border-t border-gray-100">
          <h2 className="mb-4 text-center text-3xl font-bold tracking-tight text-gray-900">
            Also included
          </h2>
          <p className="mx-auto mb-12 max-w-2xl text-center text-lg text-gray-500">
            Things we needed, so we built them.
          </p>

          {/* Bento Grid */}
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <BentoCard
              icon={Sparkles}
              title="AXEL.md"
              description="YAML frontmatter + markdown. Panes, skills, layouts—one file per project."
              className="sm:col-span-1"
            />
            <BentoCard
              icon={DollarSign}
              title="Token + cost tracking"
              description="See input/output tokens and USD per task. Cumulative totals per session."
              className="sm:col-span-1"
            />
            <BentoCard
              icon={Shield}
              title="Nothing runs without approval"
              description="Every file edit and command needs explicit permission. You set the rules."
              className="sm:col-span-1"
            />
            <BentoCard
              icon={AppleLogo}
              title="Native macOS, iOS, visionOS"
              description="SwiftUI app. Menu bar access, keyboard shortcuts, Spotlight integration."
              accent
              className="sm:col-span-2 lg:col-span-2"
            />
          </div>

          {/* Summary + CTA */}
          <div className="mt-20 text-center">
            <a
              href="https://github.com/lgalabru/axel/releases/latest/download/Axel-latest-macos.dmg"
              className="inline-flex items-center gap-2 rounded-full bg-gray-900 px-8 py-4 text-base font-semibold text-white transition-all hover:bg-gray-800 hover:scale-[1.02] active:scale-[0.98]"
            >
              <AppleLogo className="h-5 w-5" />
              Download for Mac
            </a>
          </div>
        </section>

        {/* Hero Screenshot - at bottom of page */}
        <div className="mx-auto max-w-5xl overflow-hidden">
          <div className="ml-auto w-[95%]">
            <img
              src="/hero.png"
              alt="Axel app showing task management interface"
              className="w-full h-auto"
              width={1200}
              height={800}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
