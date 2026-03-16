import { useState, type FormEvent } from "react";
import { toast } from "sonner";
import { Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/hooks/useAuth";
import { updateProfile, updateEmail, updatePassword } from "@/api/auth";
import { ApiRequestError } from "@/api/client";

function UsernameSection() {
  const { user, refresh } = useAuth();
  const [username, setUsername] = useState(user?.username ?? "");
  const [submitting, setSubmitting] = useState(false);
  const [saved, setSaved] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  function validate(): boolean {
    const e: Record<string, string> = {};
    if (!username.trim()) e.username = "Username is required";
    else if (username.length < 3 || username.length > 30)
      e.username = "Username must be between 3 and 30 characters";
    setErrors(e);
    return Object.keys(e).length === 0;
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!validate()) return;

    setSubmitting(true);
    try {
      await updateProfile({ username });
      await refresh();
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      toast.error(
        err instanceof ApiRequestError ? err.message : "Something went wrong",
      );
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form noValidate onSubmit={handleSubmit} className="grid gap-4">
      <div className="grid gap-2">
        <Label htmlFor="username">Username</Label>
        <Input
          id="username"
          value={username}
          onChange={(e) => {
            setUsername(e.target.value);
            setErrors({});
          }}
          aria-invalid={!!errors.username || undefined}
        />
        {errors.username ? (
          <p className="text-sm text-destructive">{errors.username}</p>
        ) : (
          <p className="text-sm text-muted-foreground">3–30 characters</p>
        )}
      </div>
      <Button type="submit" disabled={submitting || saved} className="w-fit">
        {submitting ? "Saving..." : saved ? <><Check className="mr-1.5 size-3.5" />Saved</> : "Save username"}
      </Button>
    </form>
  );
}

function EmailSection() {
  const { user, refresh } = useAuth();
  const [newEmail, setNewEmail] = useState(user?.email ?? "");
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [saved, setSaved] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  function validate(): boolean {
    const e: Record<string, string> = {};
    if (!newEmail.trim()) e.newEmail = "Email is required";
    if (!password) e.password = "Current password is required";
    setErrors(e);
    return Object.keys(e).length === 0;
  }

  function clearError(field: string) {
    setErrors((p) => {
      const next = { ...p };
      delete next[field];
      return next;
    });
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!validate()) return;

    setSubmitting(true);
    try {
      await updateEmail({ new_email: newEmail, current_password: password });
      await refresh();
      setPassword("");
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      toast.error(
        err instanceof ApiRequestError ? err.message : "Something went wrong",
      );
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form noValidate onSubmit={handleSubmit} className="grid gap-4">
      <div className="grid gap-2">
        <Label htmlFor="new-email">New email</Label>
        <Input
          id="new-email"
          type="email"
          value={newEmail}
          onChange={(e) => {
            setNewEmail(e.target.value);
            clearError("newEmail");
          }}
          aria-invalid={!!errors.newEmail || undefined}
        />
        {errors.newEmail && (
          <p className="text-sm text-destructive">{errors.newEmail}</p>
        )}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="email-password">Current password</Label>
        <Input
          id="email-password"
          type="password"
          autoComplete="current-password"
          value={password}
          onChange={(e) => {
            setPassword(e.target.value);
            clearError("password");
          }}
          aria-invalid={!!errors.password || undefined}
        />
        {errors.password && (
          <p className="text-sm text-destructive">{errors.password}</p>
        )}
      </div>
      <Button type="submit" disabled={submitting || saved} className="w-fit">
        {submitting ? "Saving..." : saved ? <><Check className="mr-1.5 size-3.5" />Saved</> : "Save email"}
      </Button>
    </form>
  );
}

function PasswordSection() {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [saved, setSaved] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  function validate(): boolean {
    const e: Record<string, string> = {};
    if (!currentPassword) e.currentPassword = "Current password is required";
    if (!newPassword) e.newPassword = "New password is required";
    else if (newPassword.length < 8) e.newPassword = "Password must be at least 8 characters";
    if (!confirmPassword) e.confirmPassword = "Please confirm your new password";
    else if (newPassword !== confirmPassword) e.confirmPassword = "Passwords do not match";
    setErrors(e);
    return Object.keys(e).length === 0;
  }

  function clearError(field: string) {
    setErrors((p) => {
      const next = { ...p };
      delete next[field];
      return next;
    });
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!validate()) return;

    setSubmitting(true);
    try {
      await updatePassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      toast.error(
        err instanceof ApiRequestError ? err.message : "Something went wrong",
      );
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form noValidate onSubmit={handleSubmit} className="grid gap-4">
      <div className="grid gap-2">
        <Label htmlFor="current-password">Current password</Label>
        <Input
          id="current-password"
          type="password"
          autoComplete="current-password"
          value={currentPassword}
          onChange={(e) => {
            setCurrentPassword(e.target.value);
            clearError("currentPassword");
          }}
          aria-invalid={!!errors.currentPassword || undefined}
        />
        {errors.currentPassword && (
          <p className="text-sm text-destructive">{errors.currentPassword}</p>
        )}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="new-password">New password</Label>
        <Input
          id="new-password"
          type="password"
          autoComplete="new-password"
          value={newPassword}
          onChange={(e) => {
            setNewPassword(e.target.value);
            clearError("newPassword");
          }}
          aria-invalid={!!errors.newPassword || undefined}
        />
        {errors.newPassword ? (
          <p className="text-sm text-destructive">{errors.newPassword}</p>
        ) : (
          <p className="text-sm text-muted-foreground">At least 8 characters</p>
        )}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="confirm-new-password">Confirm new password</Label>
        <Input
          id="confirm-new-password"
          type="password"
          autoComplete="new-password"
          value={confirmPassword}
          onChange={(e) => {
            setConfirmPassword(e.target.value);
            clearError("confirmPassword");
          }}
          aria-invalid={!!errors.confirmPassword || undefined}
        />
        {errors.confirmPassword && (
          <p className="text-sm text-destructive">{errors.confirmPassword}</p>
        )}
      </div>
      <Button type="submit" disabled={submitting || saved} className="w-fit">
        {submitting ? "Saving..." : saved ? <><Check className="mr-1.5 size-3.5" />Saved</> : "Change password"}
      </Button>
    </form>
  );
}

export function SettingsPage() {
  return (
    <div className="mx-auto max-w-lg space-y-8">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">Manage your account</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Username</CardTitle>
          <CardDescription>
            This is your public display name on the leaderboard
          </CardDescription>
        </CardHeader>
        <CardContent>
          <UsernameSection />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Email</CardTitle>
          <CardDescription>
            Update the email address associated with your account
          </CardDescription>
        </CardHeader>
        <CardContent>
          <EmailSection />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Password</CardTitle>
          <CardDescription>
            Change your password — this will sign you out on other devices
          </CardDescription>
        </CardHeader>
        <CardContent>
          <PasswordSection />
        </CardContent>
      </Card>
    </div>
  );
}
