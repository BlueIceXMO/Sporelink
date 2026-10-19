import React from "react";
import { cn } from "@/lib/utils";

type IconButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  active?: boolean;
  size?: "sm" | "md" | "lg";
};

export function IconButton({ label, active, size = "md", className, children, ...rest }: IconButtonProps) {
  return (
    <button
      type="button"
      className={cn("icon-button", `icon-button-${size}`, active && "is-active", className)}
      title={label}
      aria-label={label}
      {...rest}
    >
      {children}
    </button>
  );
}

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: "sm" | "md";
};

export function Button({ variant = "secondary", size = "md", className, children, ...rest }: ButtonProps) {
  return (
    <button type="button" className={cn("button", `button-${variant}`, `button-${size}`, className)} {...rest}>
      {children}
    </button>
  );
}
