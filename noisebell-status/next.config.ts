import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  experimental: {
    useCache: true,
    cacheComponents: true,
  },
};

export default nextConfig;
