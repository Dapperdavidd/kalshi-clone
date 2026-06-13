import { useState, type FormEvent } from "react";
import { useNavigate, Navigate, useSearchParams } from "react-router-dom";
import { login, signup } from "../api/endpoints";
import { ApiError } from "../api/client";
import { useAuth } from "../auth/AuthContext";
import GoogleButton from "../components/GoogleButton";

export default function LoginPage() {
  const [params] = useSearchParams();
  const [mode, setMode] = useState<"login" | "signup">(
    params.get("mode") === "signup" ? "signup" : "login",
  );
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const { loginWithToken, isLoggedIn } = useAuth();
  const navigate = useNavigate();

  if (isLoggedIn) return <Navigate to="/" replace />;

  async function handleSubmit(e: FormEvent) {
    e.preventDefault(); // stop the browser's default full-page submit
    setError(null);
    setBusy(true);
    try {
      if (mode === "signup") {
        // Create the account, then immediately log in to get a token.
        await signup(email, password);
      }
      const { token } = await login(email, password);
      loginWithToken(token); // store + update context
      navigate("/"); // go to markets
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "something went wrong");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="card col" style={{ maxWidth: 360, margin: "40px auto" }}>
      <h1 style={{ margin: 0 }}>{mode === "login" ? "Log in" : "Sign up"}</h1>

      <form onSubmit={handleSubmit} className="col">
        <input
          type="email"
          placeholder="you@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
          className="input"
        />
        <input
          type="password"
          placeholder="password (min 8 chars)"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          minLength={8}
          className="input"
        />
        <button type="submit" disabled={busy} className="btn btn-primary btn-block">
          {busy ? "..." : mode === "login" ? "Log in" : "Create account"}
        </button>
      </form>

      {error && <p className="error">{error}</p>}

      <p className="dim" style={{ margin: 0 }}>
        {mode === "login" ? "No account?" : "Have an account?"}{" "}
        <button
          onClick={() => {
            setMode(mode === "login" ? "signup" : "login");
            setError(null);
          }}
          style={{ background: "none", border: "none", color: "var(--accent)", cursor: "pointer" }}
        >
          {mode === "login" ? "Sign up" : "Log in"}
        </button>
      </p>

      <hr style={{ width: "100%", borderColor: "var(--border)" }} />
      <GoogleButton />
    </div>
  );
}
