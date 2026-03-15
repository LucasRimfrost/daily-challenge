import { useState, type FormEvent } from "react";
import { Link, Navigate, useLocation, useNavigate } from "react-router-dom";
import { Zap } from "lucide-react";
import { toast } from "sonner";
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
import { ApiRequestError } from "@/api/client";

export function RegisterPage() {
  const { user, register } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const redirectTo = (location.state as { from?: string } | null)?.from ?? "/";
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);

  if (user) return <Navigate to={redirectTo} replace />;

  function validate(): boolean {
    const e: Record<string, string> = {};
    if (!username.trim()) e.username = "Username is required";
    else if (username.length < 3 || username.length > 30) e.username = "Username must be between 3 and 30 characters";
    if (!email.trim()) e.email = "Email is required";
    if (!password) e.password = "Password is required";
    else if (password.length < 8) e.password = "Password must be at least 8 characters";
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
      await register({ username, email, password });
      navigate(redirectTo, { replace: true });
    } catch (err) {
      if (err instanceof ApiRequestError) {
        const msg = err.message.toLowerCase();
        if (msg.includes("username")) {
          setErrors({ username: err.message });
        } else if (msg.includes("email")) {
          setErrors({ email: err.message });
        } else if (msg.includes("password")) {
          setErrors({ password: err.message });
        } else {
          toast.error(err.message);
        }
      } else {
        toast.error("Something went wrong. Please try again.");
      }
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="flex min-h-[70vh] items-center justify-center px-4">
      <Card className="w-full max-w-sm">
        <CardHeader className="text-center">
          <div className="mx-auto mb-2 flex size-10 items-center justify-center rounded-lg bg-primary">
            <Zap className="size-5 text-primary-foreground" />
          </div>
          <CardTitle className="text-2xl">Create an account</CardTitle>
          <CardDescription>Start solving daily challenges</CardDescription>
        </CardHeader>
        <CardContent>
          <form noValidate onSubmit={handleSubmit} className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                autoComplete="username"
                value={username}
                onChange={(e) => { setUsername(e.target.value); clearError("username"); }}
                placeholder="coolhacker42"
                aria-invalid={!!errors.username || undefined}
              />
              {errors.username ? (
                <p className="text-sm text-destructive">{errors.username}</p>
              ) : (
                <p className="text-sm text-muted-foreground">3–30 characters</p>
              )}
            </div>
            <div className="grid gap-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                autoComplete="email"
                value={email}
                onChange={(e) => { setEmail(e.target.value); clearError("email"); }}
                placeholder="you@example.com"
                aria-invalid={!!errors.email || undefined}
              />
              {errors.email && (
                <p className="text-sm text-destructive">{errors.email}</p>
              )}
            </div>
            <div className="grid gap-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                autoComplete="new-password"
                value={password}
                onChange={(e) => { setPassword(e.target.value); clearError("password"); }}
                aria-invalid={!!errors.password || undefined}
              />
              {errors.password ? (
                <p className="text-sm text-destructive">{errors.password}</p>
              ) : (
                <p className="text-sm text-muted-foreground">At least 8 characters</p>
              )}
            </div>
            <Button type="submit" disabled={submitting} className="w-full">
              {submitting ? "Creating account..." : "Sign up"}
            </Button>
          </form>
          <p className="mt-4 text-center text-sm text-muted-foreground">
            Already have an account?{" "}
            <Link to="/login" className="text-primary underline-offset-4 hover:underline">
              Log in
            </Link>
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
