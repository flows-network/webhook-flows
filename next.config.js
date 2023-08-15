/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  async headers() {
    return [
      {
        "source": "/README.md",
        "headers": [
          { "key": "Access-Control-Allow-Origin", "value": "*" },
        ]
      }
    ]
  }
}

module.exports = nextConfig
