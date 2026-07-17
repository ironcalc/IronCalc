import {
  Button,
  type CollabProvider,
  type CollabStatus,
  colorForClient,
  decodeCursor,
  Tooltip,
} from "@ironcalc/workbook";
import { Users } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import CollabDialog from "./CollabDialog";

import "./collab.css";

// The collaboration corner of the file bar: a "Collaborate" button that
// turns the current workbook into a live session, or — once live — the
// connection status plus the avatars of everyone in the room.

export interface Collaborator {
  clientId: number;
  name: string;
  color: string;
  isSelf: boolean;
}

export function useCollaborators(provider: CollabProvider): Collaborator[] {
  const [collaborators, setCollaborators] = useState<Collaborator[]>([]);
  useEffect(() => {
    const read = () => {
      const list: Collaborator[] = [];
      for (const entry of provider.presence()) {
        const cursor = decodeCursor(entry.clientId, entry.state);
        if (cursor) {
          list.push({
            clientId: cursor.clientId,
            name: cursor.name,
            color: colorForClient(cursor.clientId),
            isSelf: cursor.clientId === provider.clientId,
          });
        }
      }
      // Self first, then stable by client id.
      list.sort(
        (a, b) =>
          Number(b.isSelf) - Number(a.isSelf) || a.clientId - b.clientId,
      );
      setCollaborators(list);
    };
    read();
    return provider.onPresenceChange(read);
  }, [provider]);
  return collaborators;
}

function useCollabStatus(provider: CollabProvider): CollabStatus {
  const [status, setStatus] = useState<CollabStatus>(provider.status);
  useEffect(() => {
    setStatus(provider.status);
    return provider.onStatusChange(setStatus);
  }, [provider]);
  return status;
}

const MAX_AVATARS = 5;

function LiveControls(properties: {
  provider: CollabProvider;
  onOpenDialog: () => void;
}) {
  const { provider, onOpenDialog } = properties;
  const { t } = useTranslation();
  const status = useCollabStatus(provider);
  const collaborators = useCollaborators(provider);
  const statusLabel = t(`file_bar.collab.${status}`);
  const shown = collaborators.slice(0, MAX_AVATARS);
  const overflow = collaborators.length - shown.length;
  return (
    <Button
      type="button"
      variant="secondary"
      className="app-ic-collab-live-button"
      onClick={onOpenDialog}
      aria-label={t("file_bar.collab.button")}
    >
      <Tooltip title={statusLabel}>
        <span
          className={`app-ic-collab-status-dot app-ic-collab-status-dot--${status}`}
        />
      </Tooltip>
      <span className="app-ic-collab-avatars">
        {shown.map((collaborator) => (
          <Tooltip
            key={collaborator.clientId}
            title={
              collaborator.isSelf
                ? `${collaborator.name} (${t("file_bar.collab.you")})`
                : collaborator.name
            }
          >
            <span
              className="app-ic-collab-avatar"
              style={{ backgroundColor: collaborator.color }}
            >
              {(collaborator.name[0] || "?").toUpperCase()}
            </span>
          </Tooltip>
        ))}
        {overflow > 0 && (
          <span className="app-ic-collab-avatar app-ic-collab-avatar--overflow">
            +{overflow}
          </span>
        )}
      </span>
    </Button>
  );
}

export function CollabControls(properties: {
  provider: CollabProvider | null;
  onStartCollaboration: () => void;
}) {
  const { provider, onStartCollaboration } = properties;
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const { t } = useTranslation();

  return (
    <div className="app-ic-collab-controls">
      {provider ? (
        <LiveControls
          provider={provider}
          onOpenDialog={() => setIsDialogOpen(true)}
        />
      ) : (
        <Button
          type="button"
          variant="secondary"
          startIcon={<Users />}
          className="app-ic-collab-start-button"
          onClick={() => {
            onStartCollaboration();
            setIsDialogOpen(true);
          }}
          aria-label={t("file_bar.collab.button")}
        >
          <span className="app-ic-collab-start-button-text">
            {t("file_bar.collab.button")}
          </span>
        </Button>
      )}
      {isDialogOpen && provider && (
        <CollabDialog
          provider={provider}
          onClose={() => setIsDialogOpen(false)}
        />
      )}
    </div>
  );
}
