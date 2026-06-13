import { NavLink, useNavigate, useSearchParams, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";

const CATEGORIES = [
  "Trending",
  "Politics",
  "Sports",
  "Crypto",
  "Economics",
  "Culture",
  "Climate",
  "Finance",
];

export default function Nav() {
  const { isLoggedIn, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [params, setParams] = useSearchParams();

  const onMarkets = location.pathname === "/";
  const q = params.get("q") ?? "";
  const activeCat = params.get("cat") ?? "Trending";

  function handleLogout() {
    logout();
    navigate("/login");
  }

  function setSearch(value: string) {
    const next = new URLSearchParams(params);
    if (value) next.set("q", value);
    else next.delete("q");
    if (onMarkets) setParams(next, { replace: true });
    else navigate(`/?${next.toString()}`);
  }

  function setCat(cat: string) {
    const next = new URLSearchParams(params);
    if (cat === "Trending") next.delete("cat");
    else next.set("cat", cat);
    if (onMarkets) setParams(next, { replace: true });
    else navigate(`/?${next.toString()}`);
  }

  return (
    <div className="knav-wrap">
      <div className="container" style={{ padding: 0 }}>
        <div className="knav">
          <NavLink to="/" className="knav-brand">
            Kalshi
          </NavLink>
          <div className="knav-links">
            <NavLink to="/" className="knav-link" end>
              Markets
            </NavLink>
            {isLoggedIn && (
              <NavLink to="/portfolio" className="knav-link">
                Portfolio
              </NavLink>
            )}
          </div>

          <div className="knav-search">
            <input
              value={q}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Trade on anything"
            />
          </div>

          <div className="knav-right">
            {isLoggedIn ? (
              <button className="btn" onClick={handleLogout}>
                Log out
              </button>
            ) : (
              <>
                <NavLink to="/login" className="btn">
                  Log in
                </NavLink>
                <NavLink to="/login?mode=signup" className="btn btn-primary">
                  Sign up
                </NavLink>
              </>
            )}
          </div>
        </div>

        {onMarkets && (
          <div className="knav" style={{ paddingTop: 0, paddingBottom: 0 }}>
            <div className="kcats">
              {CATEGORIES.map((c) => (
                <button
                  key={c}
                  className={`kcat ${activeCat === c ? "active" : ""}`}
                  onClick={() => setCat(c)}
                >
                  {c}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
