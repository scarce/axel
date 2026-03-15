'use client';

import { AxelTypo, AxelIcon } from '@axel/ui';

export function ThemedLogo({ className }: { className?: string }) {
  return (
    <>
      {/* Dark variant: hidden in light mode, visible in dark mode */}
      <AxelTypo className={`${className} hidden dark:block`} variant="dark" />
      {/* Light variant: visible in light mode, hidden in dark mode */}
      <AxelTypo className={`${className} block dark:hidden`} variant="light" />
    </>
  );
}

export function SidebarLogo() {
  return (
    <div className="flex flex-col items-center gap-3 py-4">
      <AxelIcon className="h-[72px] w-[72px]" />
      <div className="flex flex-col items-center">
        {/* Dark variant */}
        <AxelTypo className="h-5 hidden dark:block" variant="dark" />
        {/* Light variant */}
        <AxelTypo className="h-5 block dark:hidden" variant="light" />
      </div>
    </div>
  );
}
