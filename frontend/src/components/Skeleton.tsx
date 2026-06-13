export default function Skeleton({
  height = 20,
  width = "100%",
}: {
  height?: number;
  width?: number | string;
}) {
  return <div className="skeleton" style={{ height, width, marginBottom: 8 }} />;
}
