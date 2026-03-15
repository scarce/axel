import { useCurrentFrame, useVideoConfig, interpolate } from "remotion";
import { useMemo } from "react";

type Star = {
  x: number;
  y: number;
  z: number;
  size: number;
  color: string;
};

export const Starfield = () => {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();

  // Generate stars once
  const stars = useMemo<Star[]>(() => {
    const result: Star[] = [];
    for (let i = 0; i < 200; i++) {
      result.push({
        x: (Math.random() - 0.5) * width * 3,
        y: (Math.random() - 0.5) * height * 3,
        z: Math.random() * 1000,
        size: Math.random() * 3 + 1,
        color: Math.random() > 0.7 ? "#a78bfa" : "#ffffff",
      });
    }
    return result;
  }, [width, height]);

  // Speed increases over time for "warp" effect
  const speed = interpolate(frame, [0, fps * 2, fps * 3], [2, 8, 4], {
    extrapolateRight: "clamp",
  });

  return (
    <div
      style={{
        position: "absolute",
        width: "100%",
        height: "100%",
        overflow: "hidden",
        perspective: 500,
      }}
    >
      {stars.map((star, i) => {
        // Move stars toward camera
        const z = ((star.z - frame * speed) % 1000 + 1000) % 1000;
        const scale = 500 / (z + 1);

        const screenX = star.x * scale + width / 2;
        const screenY = star.y * scale + height / 2;

        // Fade out stars that are close to camera
        const opacity = interpolate(z, [0, 100, 500, 1000], [0, 1, 0.6, 0.3]);

        // Trail effect for faster speeds
        const trailLength = speed > 5 ? (speed - 5) * 20 : 0;

        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: screenX,
              top: screenY,
              width: star.size * scale,
              height: star.size * scale + trailLength,
              borderRadius: "50%",
              background:
                trailLength > 0
                  ? `linear-gradient(to bottom, transparent, ${star.color})`
                  : star.color,
              opacity,
              boxShadow:
                speed > 5
                  ? `0 0 ${scale * 4}px ${star.color}`
                  : `0 0 ${scale * 2}px ${star.color}`,
            }}
          />
        );
      })}
    </div>
  );
};
