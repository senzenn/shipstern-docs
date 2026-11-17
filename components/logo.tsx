import Image from 'next/image';

export function Logo() {
  return (
    <span className="flex items-center gap-2">
      {/* The PNG has grey outside its rounded tile; the radius clips it. */}
      <Image src="/logo.png" alt="" width={24} height={24} className="size-6 rounded-[22%]" preload />
      <span className="text-[15px] font-semibold tracking-tight">Shipstern</span>
      <span className="rounded-md border border-fd-border px-1.5 py-px text-[11px] font-medium text-fd-muted-foreground">
        Docs
      </span>
    </span>
  );
}
