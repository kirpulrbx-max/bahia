import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { Card, Row } from "./controls";
import type { Texts } from "./i18n";

interface Account {
  id: string;
  name: string;
  signedIn: boolean;
  installed: boolean;
}

/**
 * The AI accounts the menu bar reads plan usage from. Signing in runs each
 * tool's own login in a terminal: Bahia never asks for a password or key.
 */
export function AiAccountsCard({ t }: { t: Texts }) {
  const [accounts, setAccounts] = useState<Account[] | null>(null);
  const [failed, setFailed] = useState(false);
  const refresh = useCallback(() => {
    invoke<Account[]>("ai_accounts").then(setAccounts).catch(() => setFailed(true));
  }, []);
  useEffect(() => {
    refresh();
    // A login finishes in its terminal: check again when Settings comes back.
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  const signIn = (id: string) => {
    invoke("ai_sign_in", { id }).then(() => setFailed(false)).catch(() => setFailed(true));
  };

  return (
    <Card title={t.aiAccounts}>
      <p className="card-note row-hint">{t.aiAccountsHint}</p>
      {(accounts ?? []).map((account) => (
        <Row
          key={account.id}
          label={account.name}
          hint={!account.installed ? t.aiNotInstalled : account.signedIn ? t.aiSignedIn : t.aiSignedOut}
        >
          <span className="account-control">
            <span className="account-dot" data-state={account.signedIn ? "on" : account.installed ? "off" : "missing"} aria-hidden="true" />
            <button type="button" className={account.signedIn ? "push" : "push push-primary"} onClick={() => signIn(account.id)}>
              {!account.installed ? t.aiInstall : account.signedIn ? t.aiSignInAgain : t.aiSignIn}
            </button>
          </span>
        </Row>
      ))}
      {failed ? <p className="card-note error" role="alert">{t.aiError}</p> : null}
    </Card>
  );
}
