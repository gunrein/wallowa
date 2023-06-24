import forms from '@tailwindcss/forms';
import skeleton from '@skeletonlabs/skeleton/tailwind/skeleton.cjs';

/** @type {import('tailwindcss').Config}*/
const config = {
	darkMode: 'class',
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		require('path').join(require.resolve('@skeletonlabs/skeleton'), '../**/*.{html,js,svelte,ts}')
	],

	theme: {
		extend: {}
	},

	plugins: [forms, ...skeleton()]
};

module.exports = config;
