import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  async redirects() {
    return [
      // The docs open on the Get Started tab.
      { source: '/', destination: '/docs/get-started', permanent: false },
      { source: '/docs', destination: '/docs/get-started', permanent: false },
    ];
  },
};

export default withMDX(config);
