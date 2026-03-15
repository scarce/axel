import { AbsoluteFill } from "remotion";
import { Starfield } from "../components/Starfield";

export const WaitScene = () => {
  return (
    <AbsoluteFill
      style={{
        backgroundColor: "#050510",
      }}
    >
      <Starfield />
    </AbsoluteFill>
  );
};
