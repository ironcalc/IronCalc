import { Share2 } from "lucide-react";

export function ShareButton(properties: { onClick: () => void }) {
  const { onClick } = properties;
  return (
    <div
      onClick={onClick}
      onKeyDown={() => {}}
      style={{
        // position: "absolute",
        // right: "0px",
        cursor: "pointer",
        color: "#FFFFFF",
        background: "#F2994A",
        padding: "0px 10px",
        height: "36px",
        lineHeight: "36px",
        borderRadius: "4px",
        marginRight: "10px",
        display: "flex",
        alignItems: "center",
      }}
    >
      <Share2 style={{ width: "16px", height: "16px", marginRight: "10px" }} />
      <span>Share</span>
    </div>
  );
}
