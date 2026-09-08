import { Input } from "@gateway/ui";

const DIALS = ["+255","+1","+44","+91","+33","+49","+81","+86"];

export function PhoneField({ value, onChange, placeholder="712345678" }: { value: string; onChange: (v:string)=>void; placeholder?: string }) {
  // value stored as "+255 712345678" — split on space
  const [dial, ...rest] = (value || "").split(" ");
  const number = rest.join(" ");
  const isDial = dial?.startsWith("+");
  const currentDial = isDial ? dial : "+255";
  const currentNumber = isDial ? number : value || "";
  return (
    <div className="flex gap-2">
      <select
        value={currentDial}
        onChange={(e)=> onChange(`${e.target.value} ${currentNumber}`.trim())}
        className="w-[90px] rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-2 py-2 text-sm"
      >
        {DIALS.map(d=> <option key={d} value={d}>{d}</option>)}
      </select>
      <Input value={currentNumber} onChange={(e)=> onChange(`${currentDial} ${e.target.value}`.trim())} placeholder={placeholder} className="flex-1" />
    </div>
  );
}
