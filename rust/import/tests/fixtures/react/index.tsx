import cn from "classnames";
import styles from "./NoteCard.module.sass";

// Written for task-71 (not copied from any design source): the shape of a
// glass card (overlay, inner, preview slot, details) with a text field and a
// button, so the gate runs the react importer without the external corpus.

type NoteCardProps = {
    title: string;
    content: string;
    pinned?: boolean;
    onReply: (text: string) => void;
    onOpen: () => void;
    children: React.ReactNode;
};

const NoteCard = ({ title, content, pinned, onReply, onOpen, children }: NoteCardProps) => (
    <div className={styles.card} role="article">
        <div className={styles.overlay}></div>
        <div className={styles.inner}>
            <div className={cn(styles.preview, { [styles.previewPinned]: pinned })}>
                {children}
            </div>
            <div className={styles.details}>
                <div className={styles.meta}>Field note</div>
                <div className={styles.title}>{title}</div>
                <div className={styles.content}>{content}</div>
                <input className={styles.reply} placeholder="Reply" onChange={onReply} />
                <button className={styles.button} onClick={onOpen}>
                    <span className={styles.buttonTitle}>Open</span>
                    <span className={styles.buttonGlow}></span>
                </button>
            </div>
        </div>
    </div>
);

export default NoteCard;
