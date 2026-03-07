# NAIRI Frontend

React + TypeScript frontend scaffold for NAIRI.

## Run

```bash
npm install
npm run dev
```

Set backend URL (optional):

```bash
VITE_API_BASE_URL=http://localhost:8080 npm run dev
```

The UI expects backend cookie-session auth.
If you are not using same-origin proxying, ensure `VITE_API_BASE_URL` points to your backend OAuth-enabled host.
