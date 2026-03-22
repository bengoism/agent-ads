import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";

export type TocItem = { id: string; label: string };

type TocState = {
  items: TocItem[];
  setItems: (items: TocItem[]) => void;
};

const TocContext = createContext<TocState>({ items: [], setItems: () => {} });

export function TocProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<TocItem[]>([]);
  return (
    <TocContext.Provider value={{ items, setItems }}>
      {children}
    </TocContext.Provider>
  );
}

export function useToc(items: TocItem[]) {
  const { setItems } = useContext(TocContext);
  useEffect(() => {
    setItems(items);
    return () => setItems([]);
  }, [items, setItems]);
}

export function useTocItems() {
  return useContext(TocContext).items;
}
