import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
} from "remotion";
import { Starfield } from "../components/Starfield";

const AppleLogo = () => (
  <svg
    width="16"
    height="20"
    viewBox="0 0 384 512"
    fill="currentColor"
  >
    <path d="M318.7 268.7c-.2-36.7 16.4-64.4 50-84.8-18.8-26.9-47.2-41.7-84.7-44.6-35.5-2.8-74.3 20.7-88.5 20.7-15 0-49.4-19.7-76.4-19.7C63.3 141.2 4 184.8 4 273.5q0 39.3 14.4 81.2c12.8 36.7 59 126.7 107.2 125.2 25.2-.6 43-17.9 75.8-17.9 31.8 0 48.3 17.9 76.4 17.9 48.6-.7 90.4-82.5 102.6-119.3-65.2-30.7-61.7-90-61.7-91.9zm-56.6-164.2c27.3-32.4 24.8-61.9 24-72.5-24.1 1.4-52 16.4-67.9 34.9-17.5 19.8-27.8 44.3-25.6 71.9 26.1 2 49.9-11.4 69.5-34.3z" />
  </svg>
);

export const IntroScene = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Typewriter effect for "Accelerate" -> "Axel"
  const accelerateText = "Accelerate";
  const axelText = "Axel";

  // Phase 1: Type "Accelerate" (0-1.5s)
  const typeProgress = interpolate(frame, [0, fps * 1.2], [0, 1], {
    extrapolateRight: "clamp",
  });
  const charsToShow = Math.floor(typeProgress * accelerateText.length);

  // Phase 2: Pause then transform to "Axel" (1.5s-2s)
  const transformStart = fps * 1.8;
  const transformProgress = interpolate(
    frame,
    [transformStart, transformStart + fps * 0.3],
    [0, 1],
    { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
  );

  // Phase 3: "Axel" glow and scale
  const glowProgress = spring({
    frame: frame - transformStart - fps * 0.3,
    fps,
    config: { damping: 15, stiffness: 100 },
  });

  // Show accelerate while typing, then transform
  const displayText =
    frame < transformStart
      ? accelerateText.slice(0, charsToShow)
      : transformProgress < 1
        ? accelerateText.slice(
            0,
            Math.max(
              4,
              Math.floor(accelerateText.length * (1 - transformProgress))
            )
          )
        : axelText;

  // Vibration effect after transform
  const vibrateX =
    frame > transformStart + fps * 0.3 && frame < transformStart + fps * 0.6
      ? Math.sin(frame * 2) * 3 * (1 - (frame - transformStart - fps * 0.3) / (fps * 0.3))
      : 0;

  // Button animation
  const buttonSpring = spring({
    frame: frame - transformStart - fps * 0.5,
    fps,
    config: { damping: 15, stiffness: 100 },
  });
  const buttonOpacity = interpolate(buttonSpring, [0, 1], [0, 1]);
  const buttonScale = interpolate(buttonSpring, [0, 1], [0.9, 1]);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: "#050510",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      <Starfield />
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          transform: `translateX(${vibrateX}px)`,
        }}
      >
        <div
          style={{
            fontFamily: "Space Grotesk, system-ui, sans-serif",
            fontSize: 180,
            fontWeight: 700,
            color: "#f0f0f5",
            letterSpacing: "-0.02em",
            textShadow:
              glowProgress > 0
                ? `0 0 ${40 * glowProgress}px rgba(124, 58, 237, ${0.8 * glowProgress}), 0 0 ${80 * glowProgress}px rgba(124, 58, 237, ${0.4 * glowProgress})`
                : "none",
            transform: `scale(${1 + glowProgress * 0.05})`,
          }}
        >
          {displayText}
          {frame < transformStart && (
            <span
              style={{
                opacity: Math.sin(frame * 0.3) > 0 ? 1 : 0,
                color: "#7c3aed",
              }}
            >
              |
            </span>
          )}
        </div>
        <div
          style={{
            marginTop: 32,
            opacity: buttonOpacity,
            transform: `scale(${buttonScale})`,
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 10,
              padding: "14px 28px",
              background: "rgba(255, 255, 255, 0.1)",
              backdropFilter: "blur(10px)",
              border: "1px solid rgba(255, 255, 255, 0.2)",
              borderRadius: 50,
              fontFamily: "Inter, system-ui, sans-serif",
              fontSize: 18,
              fontWeight: 500,
              color: "#f0f0f5",
            }}
          >
            <AppleLogo />
            <span>Available for macOS</span>
          </div>
        </div>
      </div>
    </AbsoluteFill>
  );
};
