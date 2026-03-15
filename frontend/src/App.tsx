import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppLayout } from "@/components/AppLayout";
import { ProtectedRoute } from "@/components/ProtectedRoute";
import { LoginPage } from "@/pages/LoginPage";
import { RegisterPage } from "@/pages/RegisterPage";
import { ForgotPasswordPage } from "@/pages/ForgotPasswordPage";
import { ResetPasswordPage } from "@/pages/ResetPasswordPage";
import { ChallengePage } from "@/pages/ChallengePage";
import { ArchivePage } from "@/pages/ArchivePage";
import { HistoryPage } from "@/pages/HistoryPage";
import { HubPage } from "@/pages/HubPage";
import { LeaderboardPage } from "@/pages/LeaderboardPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { useAuth } from "@/hooks/useAuth";

function App() {
  const { loading } = useAuth();

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="mx-auto mb-3 size-8 animate-spin rounded-full border-2 border-muted border-t-primary" />
      </div>
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          {/* Public */}
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
          <Route path="/forgot-password" element={<ForgotPasswordPage />} />
          <Route path="/reset-password" element={<ResetPasswordPage />} />
          <Route path="/leaderboard" element={<LeaderboardPage />} />

          {/* Protected */}
          <Route element={<ProtectedRoute />}>
            <Route path="/" element={<HubPage />} />
            <Route path="/trivia" element={<ChallengePage />} />
            <Route path="/trivia/:date" element={<ChallengePage />} />
            <Route path="/trivia/archive" element={<ArchivePage />} />
            <Route path="/trivia/history" element={<HistoryPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Route>

          {/* Fallback */}
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
