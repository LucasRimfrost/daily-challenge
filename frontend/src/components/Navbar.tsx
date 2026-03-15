import { Link, useLocation } from "react-router-dom";
import { Archive, Flame, LogOut, Moon, Settings, Sun, Trophy, History, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { useAuth } from "@/hooks/useAuth";
import { useTheme } from "@/hooks/useTheme";
import { cn } from "@/lib/utils";

function NavLink({
  to,
  children,
}: {
  to: string;
  children: React.ReactNode;
}) {
  const { pathname } = useLocation();
  const active = pathname === to;

  return (
    <Link
      to={to}
      className={cn(
        "inline-flex items-center gap-1.5 text-sm font-medium transition-colors hover:text-foreground",
        active ? "text-foreground" : "text-muted-foreground",
      )}
    >
      {children}
    </Link>
  );
}

export function Navbar() {
  const { user, logout } = useAuth();
  const { theme, toggleTheme } = useTheme();

  return (
    <header className="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-sm">
      <div className="mx-auto flex h-14 max-w-5xl items-center justify-between px-4">
        {/* Left: brand + nav */}
        <div className="flex items-center gap-6">
          <Link
            to="/"
            className="flex items-center gap-2 text-lg font-bold tracking-tight"
          >
            <Zap className="size-5 text-primary" />
            Daily Challenge
          </Link>

          <nav className="flex items-center gap-4">
            <NavLink to="/leaderboard">
              <Trophy className="size-4" />
              <span className="hidden sm:inline">Leaderboard</span>
            </NavLink>
            {user && (
              <>
                <NavLink to="/trivia/archive">
                  <Archive className="size-4" />
                  <span className="hidden sm:inline">Archive</span>
                </NavLink>
                <NavLink to="/trivia/history">
                  <History className="size-4" />
                  <span className="hidden sm:inline">History</span>
                </NavLink>
              </>
            )}
          </nav>
        </div>

        {/* Right: streak + theme + user */}
        <div className="flex items-center gap-2">
          {user && (
            <div className="mr-2 flex items-center gap-1.5 rounded-md bg-muted px-2.5 py-1 text-sm font-medium">
              <Flame className="size-4 text-orange-500" />
              <span>{user.stats.current_streak}</span>
            </div>
          )}

          <Button variant="ghost" size="icon" onClick={toggleTheme}>
            {theme === "dark" ? (
              <Sun className="size-4" />
            ) : (
              <Moon className="size-4" />
            )}
          </Button>

          {user ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="rounded-full">
                  <Avatar className="size-8">
                    <AvatarFallback className="text-xs">
                      {user.username.slice(0, 2).toUpperCase()}
                    </AvatarFallback>
                  </Avatar>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48">
                <div className="px-2 py-1.5 text-sm">
                  <p className="font-medium">{user.username}</p>
                  <p className="text-muted-foreground">{user.email}</p>
                </div>
                <DropdownMenuSeparator />
                <DropdownMenuItem asChild>
                  <Link to="/settings">
                    <Settings className="mr-2 size-4" />
                    Settings
                  </Link>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => logout()}>
                  <LogOut className="mr-2 size-4" />
                  Log out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="sm" asChild>
                <Link to="/login">Log in</Link>
              </Button>
              <Button size="sm" asChild>
                <Link to="/register">Sign up</Link>
              </Button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
}
