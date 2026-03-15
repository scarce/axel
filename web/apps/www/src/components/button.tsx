import React, { forwardRef } from 'react';

export type ButtonSize = 'sm' | 'md' | 'lg' | 'xl';
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'purple' | 'yellow';

const sizes: Record<ButtonSize, string> = {
  sm: 'px-4 py-2 text-xs leading-4 tracking-[0.24px] gap-2',
  md: 'px-4 py-3 text-xs leading-[18px] tracking-[0.24px] gap-2',
  lg: 'px-6 py-3.5 text-sm leading-5 tracking-[0.28px] gap-2',
  xl: 'p-5 text-lg leading-[22.7px] tracking-[0.36px] gap-2',
};

const variants: Record<ButtonVariant, string> = {
  primary: [
    'bg-purple-600 text-white',
    'border border-purple-500/50',
    'hover:bg-purple-500',
    'active:bg-purple-700',
    'shadow-lg shadow-purple-500/20',
  ].join(' '),
  secondary: [
    'bg-white/5 text-white/80',
    'border border-white/10',
    'hover:bg-white/10 hover:text-white',
    'active:bg-white/15',
  ].join(' '),
  ghost: [
    'bg-transparent text-white/60',
    'border border-transparent',
    'hover:text-white hover:bg-white/5',
    'active:bg-white/10',
  ].join(' '),
  purple: [
    'bg-purple-600 text-white',
    'border border-purple-500/50',
    'hover:bg-purple-500',
    'active:bg-purple-700',
    'shadow-lg shadow-purple-500/20',
  ].join(' '),
  yellow: [
    'bg-amber-500/10 text-amber-300',
    'border border-amber-500/20',
    'hover:bg-amber-500/20',
    'active:bg-amber-500/30',
  ].join(' '),
};

const baseStyles = [
  'relative isolate inline-flex items-center justify-center',
  'font-medium',
  'rounded-lg',
  'focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-500 focus-visible:ring-offset-2 focus-visible:ring-offset-[#050510]',
  'disabled:opacity-50 disabled:pointer-events-none',
  'cursor-pointer',
  'transition-[background-color,border-color,color,box-shadow] duration-150',
].join(' ');

type BaseButtonProps = {
  size?: ButtonSize;
  variant?: ButtonVariant;
  className?: string;
  children: React.ReactNode;
};

type ButtonAsButtonProps = BaseButtonProps &
  Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'className' | 'children'> & {
    href?: never;
  };

type ButtonAsLinkProps = BaseButtonProps &
  Omit<React.AnchorHTMLAttributes<HTMLAnchorElement>, 'className' | 'children'> & {
    href: string;
  };

export type ButtonProps = ButtonAsButtonProps | ButtonAsLinkProps;

export const Button = forwardRef(function Button(
  { size = 'md', variant = 'primary', className = '', children, ...props }: ButtonProps,
  ref: React.ForwardedRef<HTMLElement>
) {
  const classes = `${baseStyles} ${sizes[size]} ${variants[variant]} ${className}`;

  if ('href' in props && props.href !== undefined) {
    return (
      <a
        {...props}
        className={classes}
        ref={ref as React.ForwardedRef<HTMLAnchorElement>}
      >
        {children}
      </a>
    );
  }

  return (
    <button
      {...(props as ButtonAsButtonProps)}
      className={classes}
      ref={ref as React.ForwardedRef<HTMLButtonElement>}
    >
      {children}
    </button>
  );
});
