export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "var(--color-bg-primary)",
        surface: "var(--color-bg-secondary)",
      }
    }
  },
  plugins: [],
}
