/** @type {import('tailwindcss').Config} */
export const content = [
  "./templates/**/*.html",
  "./src-web/**/*.{js,ts,html}"
];
export const theme = {
  extend: {},
};
export const plugins = [require("daisyui")];
