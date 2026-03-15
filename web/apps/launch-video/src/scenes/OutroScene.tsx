import {
  AbsoluteFill,
  useCurrentFrame,
  useVideoConfig,
  interpolate,
  spring,
} from "remotion";

export const OutroScene = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Logo animation
  const logoSpring = spring({
    frame,
    fps,
    config: { damping: 12, stiffness: 80 },
  });
  const logoScale = interpolate(logoSpring, [0, 1], [0.5, 1]);
  const logoOpacity = interpolate(logoSpring, [0, 1], [0, 1]);

  // CTA animation
  const ctaSpring = spring({
    frame: frame - 20,
    fps,
    config: { damping: 200 },
  });
  const ctaOpacity = interpolate(ctaSpring, [0, 1], [0, 1]);
  const ctaY = interpolate(ctaSpring, [0, 1], [30, 0]);

  // Platforms animation
  const platformsSpring = spring({
    frame: frame - 35,
    fps,
    config: { damping: 200 },
  });
  const platformsOpacity = interpolate(platformsSpring, [0, 1], [0, 1]);

  // Pulsing glow effect
  const glowPulse = 0.4 + Math.sin(frame * 0.1) * 0.2;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: "#050510",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      {/* Background glow */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          width: 800,
          height: 800,
          borderRadius: "50%",
          background: `radial-gradient(circle, rgba(124, 58, 237, ${glowPulse}) 0%, transparent 50%)`,
          filter: "blur(80px)",
        }}
      />

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: 40,
        }}
      >
        {/* Logo */}
        <div
          style={{
            opacity: logoOpacity,
            transform: `scale(${logoScale})`,
          }}
        >
          <div
            style={{
              fontFamily: "Space Grotesk, system-ui, sans-serif",
              fontSize: 140,
              fontWeight: 700,
              color: "#f0f0f5",
              letterSpacing: "-0.02em",
              textShadow: `
                0 0 60px rgba(124, 58, 237, ${glowPulse}),
                0 0 120px rgba(124, 58, 237, ${glowPulse * 0.5})
              `,
            }}
          >
            Axel
          </div>
        </div>

        {/* CTA */}
        <div
          style={{
            opacity: ctaOpacity,
            transform: `translateY(${ctaY}px)`,
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 24,
          }}
        >
          <div
            style={{
              padding: "20px 48px",
              background: "linear-gradient(135deg, #7c3aed 0%, #5b21b6 100%)",
              borderRadius: 16,
              fontFamily: "Inter, system-ui, sans-serif",
              fontSize: 24,
              fontWeight: 600,
              color: "#ffffff",
              boxShadow:
                "0 20px 40px rgba(124, 58, 237, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.1)",
            }}
          >
            Download for macOS
          </div>
          <div
            style={{
              fontFamily: "Inter, system-ui, sans-serif",
              fontSize: 18,
              color: "#8888a0",
            }}
          >
            getaxel.dev
          </div>
        </div>

        {/* Platforms */}
        <div
          style={{
            opacity: platformsOpacity,
            display: "flex",
            gap: 32,
            marginTop: 20,
          }}
        >
          {["macOS", "iOS", "visionOS"].map((platform, i) => {
            const platformDelay = spring({
              frame: frame - 40 - i * 5,
              fps,
              config: { damping: 15 },
            });

            return (
              <div
                key={platform}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  opacity: platformDelay,
                  transform: `scale(${interpolate(platformDelay, [0, 1], [0.8, 1])})`,
                }}
              >
                <div
                  style={{
                    width: 8,
                    height: 8,
                    borderRadius: "50%",
                    background: "#10b981",
                    boxShadow: "0 0 8px #10b981",
                  }}
                />
                <div
                  style={{
                    fontFamily: "Inter, system-ui, sans-serif",
                    fontSize: 16,
                    fontWeight: 500,
                    color: "#a0a0b0",
                  }}
                >
                  {platform}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
};
