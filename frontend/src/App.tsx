import { BrowserRouter, Routes, Route } from "react-router-dom";
import Nav from "./components/Nav";
import ProtectedRoute from "./auth/ProtectedRoute";
import MarketsPage from "./pages/MarketsPage";
import EventPage from "./pages/EventPage";
import MarketPage from "./pages/MarketPage";
import PortfolioPage from "./pages/PortfolioPage";
import LoginPage from "./pages/LoginPage";
import CreateMarketPage from "./pages/CreateMarketPage";

export default function App() {
  return (
    <BrowserRouter>
      <Nav />
      <main className="container">
        <Routes>
          <Route path="/" element={<MarketsPage />} />
          <Route path="/events/:id" element={<EventPage />} />
          <Route path="/markets/:id" element={<MarketPage />} />
          <Route
            path="/portfolio"
            element={
              <ProtectedRoute>
                <PortfolioPage />
              </ProtectedRoute>
            }
          />
          <Route
            path="/create"
            element={
              <ProtectedRoute>
                <CreateMarketPage />
              </ProtectedRoute>
            }
          />
          <Route path="/login" element={<LoginPage />} />
          <Route path="*" element={<h1>Not found</h1>} />
        </Routes>
      </main>
    </BrowserRouter>
  );
}
