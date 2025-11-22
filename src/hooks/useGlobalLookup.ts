import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { useStores } from "../stores/RootStore";
import type { GlobalLookupTriggerPayload } from "../types";

/**
 * Hook to handle global lookup feature (Ctrl+Right-Click or keyboard shortcut).
 * 
 * Responsibilities:
 * - Listen for global-lookup-triggered events from backend
 * - Listen for capture telemetry events for debugging
 * - Route captured words to SearchCoordinator
 * - Navigate to Define page when triggered from other pages
 * - Show error feedback when no word is captured
 */
export function useGlobalLookup() {
  const rootStore = useStores();
  const navigate = useNavigate();

  useEffect(() => {
    let unlistenTrigger: UnlistenFn | undefined;
    let unlistenTelemetry: UnlistenFn | undefined;

    const registerListeners = async () => {
      try {
        // Main trigger listener
        unlistenTrigger = await listen<GlobalLookupTriggerPayload>(
          "global-lookup-triggered",
          async (event) => {
            const candidate = event.payload.word?.trim();
            if (!candidate) {
              toast.error("No word detected. Select text before using the shortcut.");
              return;
            }
            await rootStore.searchCoordinator.search(candidate, {
              source: "global-lookup",
            });
            // Navigate to Define page (Home) after starting search
            navigate("/");
          },
        );

        // Telemetry listener for debugging capture statistics
        unlistenTelemetry = await listen<{
          strategy: string;
          outcome: string;
          duration_ms: number;
          metadata?: string;
        }>("global-lookup-capture-telemetry", (event) => {
          const { strategy, outcome, duration_ms, metadata } = event.payload;
          if (outcome !== "success") {
            console.warn(
              `[Global Lookup] ${strategy} ${outcome} (${duration_ms}ms)`,
              metadata,
            );
          } else {
            console.debug(
              `[Global Lookup] ${strategy} ${outcome} (${duration_ms}ms)`,
            );
          }
        });
      } catch (error) {
        console.error("Failed to register global lookup listeners", error);
      }
    };

    registerListeners();

    return () => {
      if (unlistenTrigger) {
        unlistenTrigger();
      }
      if (unlistenTelemetry) {
        unlistenTelemetry();
      }
    };
  }, [rootStore.searchCoordinator]);
}
