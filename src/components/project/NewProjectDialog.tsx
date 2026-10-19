import { useEffect, useState } from "react";
import { FolderPlus, Plus } from "lucide-react";
import { Modal } from "@/components/common/Modal";
import { Button } from "@/components/common/IconButton";
import { Field } from "@/components/common/Primitives";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";

const COLORS = ["#8ee39a", "#ffad7a", "#8db3ff", "#c0a0ff", "#65e0d0", "#ffd166", "#c94f71", "#a9dd7c"];

export function NewProjectDialog() {
  const open = useAppStore((s) => s.newProjectOpen);
  const setOpen = useAppStore((s) => s.setNewProjectOpen);
  const createProject = useAppStore((s) => s.createProject);
  const folders = useAppStore((s) => s.folders);
  const { t } = useI18n();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [color, setColor] = useState(COLORS[0]);
  const [folderId, setFolderId] = useState<string>("");

  useEffect(() => {
    if (open) {
      setName("");
      setDescription("");
      setColor(COLORS[0]);
      setFolderId("");
    }
  }, [open]);

  const submit = async () => {
    if (!name.trim()) return;
    await createProject({
      name: name.trim(),
      description: description.trim() || undefined,
      color,
      folderId: folderId || null,
    });
  };

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title={t("project.new")}
      subtitle={t("welcome.subtitle")}
      width="md"
      footer={
        <>
          <Button variant="ghost" onClick={() => setOpen(false)}>
            {t("app.cancel")}
          </Button>
          <Button variant="primary" onClick={() => void submit()} disabled={!name.trim()}>
            <Plus size={16} /> {t("project.create")}
          </Button>
        </>
      }
    >
      <div className="form-stack">
        <Field label={t("project.name")}>
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder={t("project.namePlaceholder")}
            onKeyDown={(e) => {
              if (e.key === "Enter") void submit();
            }}
          />
        </Field>
        <Field label={t("project.description")}>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder={t("project.descriptionPlaceholder")}
            rows={3}
          />
        </Field>
        <div className="field-row">
          <Field label={t("project.color")}>
            <div className="color-row">
              {COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  className={`color-dot ${color === c ? "is-active" : ""}`}
                  style={{ background: c }}
                  onClick={() => setColor(c)}
                  aria-label={c}
                />
              ))}
            </div>
          </Field>
          <Field label={t("project.folder")}>
            <select value={folderId} onChange={(e) => setFolderId(e.target.value)}>
              <option value="">{t("project.noFolder")}</option>
              {folders.map((folder) => (
                <option key={folder.id} value={folder.id}>
                  {folder.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
      </div>
    </Modal>
  );
}

export function NewFolderDialog() {
  const open = useAppStore((s) => s.newFolderOpen);
  const setOpen = useAppStore((s) => s.setNewFolderOpen);
  const createFolder = useAppStore((s) => s.createFolder);
  const { t } = useI18n();
  const [name, setName] = useState("");

  useEffect(() => {
    if (open) setName("");
  }, [open]);

  const submit = async () => {
    if (!name.trim()) return;
    await createFolder(name.trim());
  };

  return (
    <Modal
      open={open}
      onClose={() => setOpen(false)}
      title={t("folder.new")}
      width="sm"
      footer={
        <>
          <Button variant="ghost" onClick={() => setOpen(false)}>
            {t("app.cancel")}
          </Button>
          <Button variant="primary" onClick={() => void submit()} disabled={!name.trim()}>
            <FolderPlus size={16} /> {t("app.create")}
          </Button>
        </>
      }
    >
      <Field label={t("folder.name")}>
        <input
          autoFocus
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t("folder.namePlaceholder")}
          onKeyDown={(e) => {
            if (e.key === "Enter") void submit();
          }}
        />
      </Field>
    </Modal>
  );
}
