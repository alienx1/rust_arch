// Tailwind CSS 3.4 configuration of master-app. trunk compiles the CSS at build time
// (`<link data-trunk rel="tailwind-css" href="tailwind.css">` in index.html, CLI version pinned in
// Trunk.toml.tmpl) from the class names found in the files listed under `content`.
//
// A class name must appear in the source as ONE complete string ("bg-primary text-white"):
// building it with format!("w-{n}") produces nothing. List such names in `safelist` instead.
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
          border: 'var(--border)',
          input: 'var(--input)',
          ring: 'var(--ring)',
          background: 'var(--background)',
          foreground: 'var(--foreground)',
          primary: {
            DEFAULT: 'var(--primary)',
            foreground: 'var(--primary-foreground)',
          },
          secondary: {
            DEFAULT: 'var(--secondary)',
            foreground: 'var(--secondary-foreground)',
          },
          destructive: {
            DEFAULT: 'oklch(0.577 0.245 27.325)',
            foreground: 'oklch(0.985 0 0)',
          },
          muted: {
            DEFAULT: 'var(--muted)',
            foreground: 'var(--muted-foreground)',
          },
          accent: {
            DEFAULT: 'var(--accent)',
            foreground: 'var(--accent-foreground)',
          },
          popover: {
            DEFAULT: 'var(--popover)',
            foreground: 'var(--popover-foreground)',
          },
          card: {
            DEFAULT: 'var(--card)',
            foreground: 'var(--card-foreground)',
          },
          sidebar: {
            DEFAULT: 'var(--sidebar)',
            foreground: 'var(--foreground)',
            primary: 'var(--sidebar-primary)',
            'primary-foreground': 'var(--sidebar-primary-foreground)',
            accent: 'var(--accent)',
            'accent-foreground': 'var(--accent-foreground)',
            border: 'var(--border)',
            ring: 'var(--ring)',
          },
        },
        borderRadius: {
          lg: 'var(--radius)',
          md: 'calc(var(--radius) - 2px)',
          sm: 'calc(var(--radius) - 4px)',
        },
        fontFamily: {
          sans: ['Inter', 'sans-serif'],
        },
      },
    },
};
