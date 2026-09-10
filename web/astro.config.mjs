// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://papermint.zerathlabs.com',
	integrations: [
		starlight({
			title: '🌿 papermint',
			description: 'High-performance, type-safe thermal receipt printing for Rust, Node.js & Mobile',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/zerathlabs/papermint' },
			],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Quickstart Guide', slug: 'getting-started/quickstart' },
					],
				},
				{
					label: 'Architecture & Design',
					items: [
						{ label: 'Core Architecture', slug: 'architecture/overview' },
						{ label: 'Hardware Protocols', slug: 'architecture/protocols' },
						{ label: 'Supported Hardware', slug: 'architecture/printers' },
					],
				},
				{
					label: 'Platforms & Ecosystem',
					items: [
						{ label: 'Node.js & TypeScript', slug: 'platforms/nodejs' },
						{ label: 'Mobile (Expo & React Native)', slug: 'platforms/mobile' },
						{ label: 'Print Daemon (HTTP REST)', slug: 'platforms/daemon' },
					],
				},
				{
					label: 'API References',
					items: [
						{ label: 'Daemon REST API', slug: 'reference/daemon-api' },
						{ label: 'C-ABI Header (papermint.h)', slug: 'reference/c-abi' },
					],
				},
			],
		}),
	],
});
