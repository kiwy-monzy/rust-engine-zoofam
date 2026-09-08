import { useState } from "react";

export interface PassField {
  key: string;
  label?: string;
  value: string | number;
  changeMessage?: string;
  textAlignment?: "PKTextAlignmentLeft" | "PKTextAlignmentCenter" | "PKTextAlignmentRight";
}

export interface PassStructure {
  headerFields?: PassField[];
  primaryFields?: PassField[];
  secondaryFields?: PassField[];
  auxiliaryFields?: PassField[];
  backFields?: PassField[];
}

export interface PKPassData {
  formatVersion?: number;
  passTypeIdentifier?: string;
  serialNumber?: string;
  teamIdentifier?: string;
  organizationName?: string;
  description?: string;
  logoText?: string;
  backgroundColor?: string;
  foregroundColor?: string;
  labelColor?: string;
  barcode?: {
    format: string;
    message: string;
    messageEncoding: string;
    altText?: string;
  };
  barcodes?: Array<{
    format: string;
    message: string;
    messageEncoding: string;
    altText?: string;
  }>;
  coupon?: PassStructure;
  eventTicket?: PassStructure;
  storeCard?: PassStructure;
  generic?: PassStructure;
  boardingPass?: PassStructure;
}

export function parseRgb(rgbStr?: string, fallback = "#1e293b"): string {
  if (!rgbStr) return fallback;
  if (rgbStr.startsWith("#")) return rgbStr;
  const match = rgbStr.match(/\d+/g);
  if (match && match.length >= 3) {
    return `rgb(${match[0]}, ${match[1]}, ${match[2]})`;
  }
  return rgbStr;
}

export function getPassStructure(pass: PKPassData): { style: string; struct: PassStructure } {
  if (pass.coupon) return { style: "Coupon", struct: pass.coupon };
  if (pass.eventTicket) return { style: "Event Ticket", struct: pass.eventTicket };
  if (pass.storeCard) return { style: "Store Card", struct: pass.storeCard };
  if (pass.boardingPass) return { style: "Boarding Pass", struct: pass.boardingPass };
  return { style: "Generic", struct: pass.generic || {} };
}

export function PassPreview({
  passData,
  qrSvg,
}: {
  passData: PKPassData;
  qrSvg?: string;
}) {
  const [side, setSide] = useState<"front" | "back">("front");

  const bgColor = parseRgb(passData.backgroundColor, "#1e293b");
  const fgColor = parseRgb(passData.foregroundColor, "#ffffff");
  const labelColor = parseRgb(passData.labelColor, "rgba(255, 255, 255, 0.7)");

  const { style, struct } = getPassStructure(passData);
  const barcodeInfo = passData.barcode || (passData.barcodes && passData.barcodes[0]);

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 12 }}>
      {/* Flip Button Bar */}
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <span style={{ fontSize: 12, fontWeight: 500, color: "var(--muted)" }}>
          Pass Preview ({style})
        </span>
        <button
          type="button"
          onClick={() => setSide(side === "front" ? "back" : "front")}
          style={{
            padding: "4px 10px",
            fontSize: 12,
            borderRadius: 6,
            border: "1px solid var(--border)",
            background: "var(--bg)",
            color: "var(--fg)",
            cursor: "pointer",
          }}
        >
          Flip to {side === "front" ? "Back 🔄" : "Front 🎴"}
        </button>
      </div>

      {/* Card Container */}
      <div
        style={{
          width: 320,
          minHeight: 460,
          borderRadius: 16,
          backgroundColor: bgColor,
          color: fgColor,
          padding: 20,
          boxShadow: "0 10px 25px -5px rgba(0,0,0,0.3), 0 8px 10px -6px rgba(0,0,0,0.2)",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          position: "relative",
          overflow: "hidden",
          transition: "all .3s ease-in-out",
          fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        }}
      >
        {side === "front" ? (
          <>
            {/* Header: Logo Text + Header Fields */}
            <div>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div style={{ fontWeight: 700, fontSize: 16, letterSpacing: -0.3 }}>
                  {passData.logoText || passData.organizationName || "Apple Wallet"}
                </div>
                {struct.headerFields && struct.headerFields.length > 0 && (
                  <div style={{ display: "flex", gap: 12, textAlign: "right" }}>
                    {struct.headerFields.map((f, i) => (
                      <div key={f.key || i}>
                        {f.label && (
                          <div style={{ fontSize: 9.5, textTransform: "uppercase", color: labelColor, fontWeight: 600 }}>
                            {f.label}
                          </div>
                        )}
                        <div style={{ fontSize: 13, fontWeight: 600 }}>{f.value}</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Primary Fields */}
              {struct.primaryFields && struct.primaryFields.length > 0 && (
                <div style={{ marginTop: 24, display: "flex", justifyContent: "space-between", gap: 8 }}>
                  {struct.primaryFields.map((f, i) => (
                    <div key={f.key || i}>
                      {f.label && (
                        <div style={{ fontSize: 10, textTransform: "uppercase", color: labelColor, fontWeight: 600 }}>
                          {f.label}
                        </div>
                      )}
                      <div style={{ fontSize: 26, fontWeight: 800, lineHeight: 1.1 }}>{f.value}</div>
                    </div>
                  ))}
                </div>
              )}

              {/* Secondary Fields */}
              {struct.secondaryFields && struct.secondaryFields.length > 0 && (
                <div style={{ marginTop: 18, display: "flex", gap: 16, flexWrap: "wrap" }}>
                  {struct.secondaryFields.map((f, i) => (
                    <div key={f.key || i} style={{ minWidth: 80 }}>
                      {f.label && (
                        <div style={{ fontSize: 9.5, textTransform: "uppercase", color: labelColor, fontWeight: 600 }}>
                          {f.label}
                        </div>
                      )}
                      <div style={{ fontSize: 13, fontWeight: 600 }}>{f.value}</div>
                    </div>
                  ))}
                </div>
              )}

              {/* Auxiliary Fields */}
              {struct.auxiliaryFields && struct.auxiliaryFields.length > 0 && (
                <div style={{ marginTop: 14, display: "flex", gap: 16, flexWrap: "wrap" }}>
                  {struct.auxiliaryFields.map((f, i) => (
                    <div key={f.key || i} style={{ minWidth: 70 }}>
                      {f.label && (
                        <div style={{ fontSize: 9.5, textTransform: "uppercase", color: labelColor, fontWeight: 600 }}>
                          {f.label}
                        </div>
                      )}
                      <div style={{ fontSize: 12.5, fontWeight: 500 }}>{f.value}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Barcode / QR Code Area */}
            <div style={{ marginTop: 24, padding: "12px", background: "#ffffff", borderRadius: 12, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center" }}>
              {qrSvg ? (
                <div
                  style={{ width: 120, height: 120, display: "flex", alignItems: "center", justifyContent: "center" }}
                  dangerouslySetInnerHTML={{ __html: qrSvg }}
                />
              ) : (
                <div style={{ width: 110, height: 110, background: "#f1f5f9", borderRadius: 8, display: "flex", alignItems: "center", justifyContent: "center", color: "#64748b", fontSize: 10, textAlign: "center", padding: 8 }}>
                  [ Barcode / QR Code ]
                </div>
              )}
              {barcodeInfo && (
                <div style={{ fontSize: 11, fontFamily: "monospace", color: "#1e293b", marginTop: 4, fontWeight: 600 }}>
                  {barcodeInfo.altText || barcodeInfo.message}
                </div>
              )}
            </div>
          </>
        ) : (
          /* Back Side */
          <div style={{ display: "flex", flexDirection: "column", gap: 14, height: "100%" }}>
            <div style={{ fontWeight: 700, fontSize: 15, borderBottom: `1px solid ${labelColor}`, paddingBottom: 8 }}>
              Pass Details & Information
            </div>
            {struct.backFields && struct.backFields.length > 0 ? (
              struct.backFields.map((f, i) => (
                <div key={f.key || i} style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  {f.label && (
                    <div style={{ fontSize: 10, textTransform: "uppercase", color: labelColor, fontWeight: 600 }}>
                      {f.label}
                    </div>
                  )}
                  <div style={{ fontSize: 12.5, lineHeight: 1.4 }}>{f.value}</div>
                </div>
              ))
            ) : (
              <div style={{ fontSize: 12, color: labelColor, fontStyle: "italic", textAlign: "center", marginTop: 40 }}>
                No back fields defined.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
