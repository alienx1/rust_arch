// Tailwind CSS 3.4 configuration of master-app. trunk compiles the CSS at build time
// (`<link data-trunk rel="tailwind-css" href="tailwind.css">` in index.html, CLI version pinned in
// Trunk.toml.tmpl) from the class names found in the files listed under `content`.
//
// A class name must appear in the source as ONE complete string ("bg-primary text-white"):
// building it with format!("w-{n}") produces nothing. List such names in `safelist` instead.

// Theme colours are CSS variables (package/style.css). Tailwind can only apply an opacity modifier
// (`bg-primary/10`, `border-border/60`) to a colour it can decompose, so wrap each variable: with a
// modifier the colour is mixed with transparent (works for oklch), without one it stays the plain var.
const v = (name) => ({ opacityValue }) =>
  // Plain utilities (no modifier) get undefined or the `var(--tw-*-opacity)` placeholder: keep the var.
  opacityValue === undefined || Number.isNaN(Number(opacityValue)) || Number(opacityValue) === 1
    ? `var(--${name})`
    : `color-mix(in oklch, var(--${name}) ${Math.round(Number(opacityValue) * 100)}%, transparent)`;

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './index.html',
    './src/**/*.rs',
    '../../package/src/**/*.rs',
  ],
  safelist: [],

    darkMode: 'class',
    theme: {
      extend: {
        colors: {
          border: v('border'),
          input: v('input'),
          ring: v('ring'),
          background: v('background'),
          foreground: v('foreground'),
          primary: {
            DEFAULT: v('primary'),
            foreground: v('primary-foreground'),
          },
          secondary: {
            DEFAULT: v('secondary'),
            foreground: v('secondary-foreground'),
          },
          destructive: {
            DEFAULT: v('destructive'),
            foreground: v('destructive-foreground'),
          },
          muted: {
            DEFAULT: v('muted'),
            foreground: v('muted-foreground'),
          },
          accent: {
            DEFAULT: v('accent'),
            foreground: v('accent-foreground'),
          },
          popover: {
            DEFAULT: v('popover'),
            foreground: v('popover-foreground'),
          },
          card: {
            DEFAULT: v('card'),
            foreground: v('card-foreground'),
          },
          sidebar: {
            DEFAULT: v('sidebar'),
            foreground: v('foreground'),
            primary: v('sidebar-primary'),
            'primary-foreground': v('sidebar-primary-foreground'),
            accent: v('accent'),
            'accent-foreground': v('accent-foreground'),
            border: v('border'),
            ring: v('ring'),
          },
        },
        borderRadius: {
          lg: 'var(--radius)',
          md: 'calc(var(--radius) - 2px)',
          sm: 'calc(var(--radius) - 4px)',
        },
        boxShadow: {
          xs: 'var(--shadow-xs)',
          sm: 'var(--shadow-sm)',
          DEFAULT: 'var(--shadow-sm)',
          md: 'var(--shadow-md)',
          lg: 'var(--shadow-lg)',
        },
        keyframes: {
          'fade-in': { from: { opacity: '0', transform: 'translateY(4px)' }, to: { opacity: '1', transform: 'none' } },
        },
        animation: { 'fade-in': 'fade-in 0.18s ease-out' },
        fontFamily: {
          sans: ['Inter', 'sans-serif'],
        },
      },
    },
};
