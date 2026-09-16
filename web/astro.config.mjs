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
			favicon: '/favicon.svg',
			customCss: ['./src/styles/custom.css'],
			head: [
				{
					tag: 'meta',
					attrs: { name: 'theme-color', content: '#0b0f19' },
				},
			],
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
						{ label: 'Desktop (Electron & Tauri)', slug: 'platforms/desktop' },
						{ label: 'React Native & Expo', slug: 'platforms/mobile' },
						{ label: 'Flutter (dart:ffi)', slug: 'platforms/flutter' },
						{ label: 'Native iOS, Android & KMP', slug: 'platforms/native-mobile' },
						{ label: 'Print Daemon (HTTP REST)', slug: 'platforms/daemon' },
						{ label: 'WebUSB (Browser Direct)', slug: 'platforms/webusb' },
					],
				},
				{
					label: 'Guides & Compliance',
					items: [
						{ label: '2D Label & Barcode Printing', slug: 'guides/label-printing' },
						{ label: 'Cloud & Remote WebSocket Printing', slug: 'guides/cloud-printing' },
						{ label: 'E-Invoicing (ZATCA & UAE FTA)', slug: 'guides/e-invoicing' },
						{ label: 'Virtual Receipt Previewer', slug: 'guides/receipt-previewer' },
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
