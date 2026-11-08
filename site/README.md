# Wren landing site

The static landing page for Wren at <https://wren.ma2.dev>. Built with
[Astro](https://astro.build) and served as a Cloudflare Worker with static
assets (no Worker script). It is separate from the app: nothing here is
compiled into Wren.

Every claim on the page comes from the repository's `README.md` and `docs/`.
When features, platforms or install steps change there, update
`src/pages/index.astro` to match. Images in `public/` are copies of
`packaging/linux/wren-256.png`, `packaging/icons/wren.svg` and
`docs/screenshots/`.

## Run locally

Needs Node.js 22 or newer.

```sh
cd site
npm install
npm run dev        # http://localhost:4321, also bound to the network (--host)
npm run build      # writes dist/
npm run preview    # serves dist/
```

## Deploy

`wrangler.jsonc` names the Worker `site-wren`, serves `dist/` and attaches
the custom domain `wren.ma2.dev` (the `ma2.dev` zone must be on the same
Cloudflare account).

```sh
cd site
export CLOUDFLARE_API_TOKEN=...   # token with Workers Scripts and Workers Routes edit
export CLOUDFLARE_ACCOUNT_ID=...
npm run deploy                    # astro build && wrangler deploy
```

Wrangler creates the DNS record for the custom domain on the first deploy.
Fonts are self-hosted through Fontsource, so the page makes no third-party
requests.
