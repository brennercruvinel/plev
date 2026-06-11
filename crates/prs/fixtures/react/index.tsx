import cn from "classnames";
import styles from "./HoffResearchCard.module.sass";

type HoffResearchCardProps = {
    title: string;
    content: string;
    borderPreview?: boolean;
    children: React.ReactNode;
};

const HoffResearchCard = ({ title, content, borderPreview, children }: HoffResearchCardProps) => (
    <div className={styles["hoff-research-card"]}>
        <div className={styles.overlay}></div>
        <div className={styles.inner}>
            <div
                className={cn(styles.preview, {
                    [styles.previewBorder]: borderPreview,
                })}
            >
                {children}
            </div>
            <div className={styles.details}>
                <div className={styles.title}>{title}</div>
                <div className={styles.content}>{content}</div>
                <button className={styles.button}>
                    <span className={styles.buttonTitle}>Discover</span>
                    <span className={styles.buttonCircle}></span>
                </button>
            </div>
        </div>
    </div>
);

export default HoffResearchCard;
