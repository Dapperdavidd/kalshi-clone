import { GoogleLogin } from "@react-oauth/google";
import { useNavigate } from "react-router-dom";
import { googleLogin } from "../api/endpoints";
import { ApiError } from "../api/client";
import { useAuth } from "../auth/AuthContext";
import { useState } from "react";

export default function GoogleButton() {
  const { loginWithToken } = useAuth();
  const navigate = useNavigate();
  const [error, setError] = useState<string | null>(null);

  return (
    <div>
      <GoogleLogin
        onSuccess={async (credentialResponse) => {
          const credential = credentialResponse.credential;
          if (!credential) {
            setError("no credential from Google");
            return;
          }
          try {
            // Send Google's ID token to OUR backend; get OUR JWT back.
            const { token } = await googleLogin(credential);
            loginWithToken(token);
            navigate("/");
          } catch (err) {
            setError(err instanceof ApiError ? err.message : "google sign-in failed");
          }
        }}
        onError={() => setError("google sign-in was cancelled or failed")}
      />
      {error && <p className="error">{error}</p>}
    </div>
  );
}
