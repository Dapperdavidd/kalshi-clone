# Kalshi-clone frontend

Vite + React + TypeScript SPA for the kalshi-clone prediction market. Talks to
the Rust backend over REST + WebSocket.

## Setup

```bash
npm install
npm run dev      # http://localhost:5173
```

The backend must be running (default `http://127.0.0.1:8080`) for data to load.

## Required environment variables (`.env`)

| Variable                | Purpose                                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------------------- |
| `VITE_API_URL`          | Backend base URL. Dev: `http://127.0.0.1:8080`. The WS URL is derived by swapping `http`→`ws`.  |
| `VITE_GOOGLE_CLIENT_ID` | Public Google OAuth client ID (same one the backend verifies).                                  |

> `.env` is git-ignored. Only `VITE_`-prefixed vars are exposed to the browser
> bundle, so no server secret can leak in.

## Scripts

- `npm run dev` — dev server with HMR
- `npm run build` — typecheck (`tsc -b`) + production build to `dist/`
- `npm run preview` — serve the production build locally

## Structure

```
src/
├── api/          typed client (client.ts), endpoint SDK (endpoints.ts), types.ts
├── auth/         AuthContext (JWT in localStorage), ProtectedRoute
├── components/   Nav, OrderBook, TradesTape, PriceChart, OrderTicket, ResolveControls, Toast, ...
├── pages/        MarketsPage, MarketPage, PortfolioPage, LoginPage
├── lib/          format.ts, useAsync.ts (fetch hook), useMarketSocket.ts (live WS)
├── App.tsx       routes
└── main.tsx      providers (Google OAuth → Auth → Toast → App)
```
