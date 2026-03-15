import {
  AbsoluteFill,
  useVideoConfig,
} from "remotion";
import { TransitionSeries, linearTiming } from "@remotion/transitions";
import { fade } from "@remotion/transitions/fade";
import { IntroScene } from "./scenes/IntroScene";
import { WaitScene } from "./scenes/WaitScene";
import { OutroScene } from "./scenes/OutroScene";

export const LaunchVideo = () => {
  const { fps } = useVideoConfig();

  return (
    <AbsoluteFill style={{ backgroundColor: "#050510" }}>
      <TransitionSeries>
        {/* Intro - Logo animation (3.5s) */}
        <TransitionSeries.Sequence durationInFrames={Math.round(3.5 * fps)}>
          <IntroScene />
        </TransitionSeries.Sequence>

        <TransitionSeries.Transition
          presentation={fade()}
          timing={linearTiming({ durationInFrames: Math.round(0.5 * fps) })}
        />

        {/* Wait / Middle section (10s) */}
        <TransitionSeries.Sequence durationInFrames={Math.round(10 * fps)}>
          <WaitScene />
        </TransitionSeries.Sequence>

        <TransitionSeries.Transition
          presentation={fade()}
          timing={linearTiming({ durationInFrames: Math.round(0.5 * fps) })}
        />

        {/* Outro with CTA (3s) */}
        <TransitionSeries.Sequence durationInFrames={Math.round(3 * fps)}>
          <OutroScene />
        </TransitionSeries.Sequence>
      </TransitionSeries>
    </AbsoluteFill>
  );
};
