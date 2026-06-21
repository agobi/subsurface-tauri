// AI-generated (Claude)
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { app } from "$lib/stores/app.svelte.ts";
import CloudLoginDialog from "$lib/components/CloudLoginDialog.svelte";
import sample from "$lib/fixtures/logbook.sample.json";
import type { Logbook } from "$lib/types.ts";

describe("CloudLoginDialog", () => {
  beforeEach(() => app.reset());

  it("renders email and password fields and action buttons", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null); // get_cloud_credentials
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess: vi.fn() } });
    await waitFor(() => {
      expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    });
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /open cloud/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeInTheDocument();
  });

  it("prefills email from get_cloud_credentials", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("saved@example.com"); // get_cloud_credentials
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess: vi.fn() } });
    await waitFor(() => {
      expect(screen.getByLabelText(/email/i)).toHaveValue("saved@example.com");
    });
  });

  it("leaves email empty when get_cloud_credentials returns null", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess: vi.fn() } });
    await waitFor(() => {
      expect(screen.getByLabelText(/email/i)).toHaveValue("");
    });
  });

  it("calls onClose when Cancel is clicked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);
    const onClose = vi.fn();
    render(CloudLoginDialog, { props: { onClose, onSuccess: vi.fn() } });
    await waitFor(() => screen.getByRole("button", { name: /cancel/i }));
    await fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("calls onSuccess with email after successful open", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null); // get_cloud_credentials
    vi.mocked(invoke).mockResolvedValueOnce(sample as unknown as Logbook); // open_cloud_logbook
    const onSuccess = vi.fn();
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess } });
    await waitFor(() => screen.getByLabelText(/email/i));
    await fireEvent.input(screen.getByLabelText(/email/i), { target: { value: "user@example.com" } });
    await fireEvent.input(screen.getByLabelText(/password/i), { target: { value: "secret" } });
    await fireEvent.click(screen.getByRole("button", { name: /open cloud/i }));
    await waitFor(() => expect(onSuccess).toHaveBeenCalledWith("user@example.com"));
  });

  it("shows error message when open_cloud_logbook fails", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null); // get_cloud_credentials
    vi.mocked(invoke).mockRejectedValueOnce("Authentication failed. Check your email and password.");
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess: vi.fn() } });
    await waitFor(() => screen.getByLabelText(/email/i));
    await fireEvent.input(screen.getByLabelText(/email/i), { target: { value: "bad@example.com" } });
    await fireEvent.input(screen.getByLabelText(/password/i), { target: { value: "wrong" } });
    await fireEvent.click(screen.getByRole("button", { name: /open cloud/i }));
    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent(/Authentication failed/i)
    );
  });

  it("disables the Open Cloud button while loading", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null); // get_cloud_credentials
    // open_cloud_logbook never resolves (simulates slow network)
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(() => {}));
    render(CloudLoginDialog, { props: { onClose: vi.fn(), onSuccess: vi.fn() } });
    await waitFor(() => screen.getByLabelText(/email/i));
    await fireEvent.input(screen.getByLabelText(/email/i), { target: { value: "u@e.com" } });
    await fireEvent.input(screen.getByLabelText(/password/i), { target: { value: "pw" } });
    await fireEvent.click(screen.getByRole("button", { name: /open cloud/i }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /opening/i })).toBeDisabled()
    );
  });
});
