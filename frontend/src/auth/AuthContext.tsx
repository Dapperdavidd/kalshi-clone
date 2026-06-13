import { createContext, useContext, useState, type ReactNode } from "react";
import { getToken, setToken, clearToken } from "../api/client";

interface Claims {
  sub: number;
  exp: number;
  is_admin: boolean;
}

/** Decode (NOT verify) a JWT payload for display/routing. Returns null if the
 *  token is missing, malformed, or expired. */
function decodeToken(token: string | null): Claims | null {
  if (!token) return null;
  try {
    const payload = JSON.parse(atob(token.split(".")[1]));
    if (typeof payload.exp === "number" && payload.exp * 1000 < Date.now()) {
      return null; // expired — treat as logged out
    }
    return payload as Claims;
  } catch {
    return null;
  }
}

interface AuthState {
  userId: number | null;
  isAdmin: boolean;
  isLoggedIn: boolean;
  loginWithToken: (token: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthState | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  // Seed from localStorage so a refresh keeps you logged in.
  const [claims, setClaims] = useState<Claims | null>(() => decodeToken(getToken()));

  const loginWithToken = (token: string) => {
    setToken(token);
    setClaims(decodeToken(token));
  };

  const logout = () => {
    clearToken();
    setClaims(null);
  };

  const value: AuthState = {
    userId: claims?.sub ?? null,
    isAdmin: claims?.is_admin ?? false,
    isLoggedIn: claims !== null,
    loginWithToken,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/** Hook every component uses to read auth state. */
export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within <AuthProvider>");
  return ctx;
}
