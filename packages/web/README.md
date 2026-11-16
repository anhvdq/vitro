# @vitro/web

SolidJS app (Vite, TanStack Query, `@solidjs/router`) that replaces `packages/frontend` by P3. It currently has one slice: sign in, show the current user, sign out.

```shell
bun install
just setup        # creates packages/web/.env from .env.example
just dev-api      # the API on :8000
just dev-web      # this app on :5174
```

- The API client is generated: run `just contracts` after changing API routes or schemas. Do not edit `src/api/schema.d.ts` by hand.
- Visible strings live in `packages/contracts/i18n/{vi,en}.json`; Vietnamese is the default.
- The session token is kept in `sessionStorage` and sent as a Bearer header. This is temporary: an HttpOnly cookie with CSRF protection replaces it, and until then scripts on the page can read the token.
- Verify with `just check-web` and `just test-web`.
