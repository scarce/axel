import { Composition } from "remotion";
import { LaunchVideo } from "./LaunchVideo";

// Duration calculation:
// Intro: 3.5s + Wait: 10s + Outro: 3s = 16.5s
// Minus transitions overlap: 2 × 0.5s = 1s
// Total: 15.5s = 465 frames at 30fps

export const RemotionRoot = () => {
  return (
    <Composition
      id="LaunchVideo"
      component={LaunchVideo}
      durationInFrames={465}
      fps={30}
      width={1920}
      height={1080}
    />
  );
};
