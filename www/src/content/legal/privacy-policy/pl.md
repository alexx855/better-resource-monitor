---
title: "Polityka prywatności | Better Resource Monitor"
description: "Dowiedz się, jak Better Resource Monitor dba o prywatność: bez zbierania danych osobowych, bez analityki, bez telemetrii i z monitorowaniem systemu wyłącznie lokalnie."
ogImage: "/images/og-privacy-pl.png"
---

# Polityka prywatności

**Ostatnia aktualizacja: 30 października 2026**

## Informacje ogólne

Better Resource Monitor („Aplikacja”) to aplikacja na komputer, która wyświetla informacje o zasobach systemu na pasku menu. Niniejsza polityka prywatności wyjaśnia, jak postępujemy z Twoimi danymi.

## Zbieranie danych

**Nie zbieramy żadnych danych osobowych.**

Aplikacja działa w całości lokalnie na Twoim urządzeniu. Wszystkie informacje o systemie (użycie CPU, pamięci i dysku, aktywność sieciowa, statystyki GPU) są:
- odczytywane bezpośrednio z systemu operacyjnego,
- wyświetlane wyłącznie na Twoim ekranie,
- nigdy nieprzesyłane na żadne zewnętrzne serwery,
- nigdy niezapisywane w żadnej zewnętrznej bazie danych.

Użycie dysku jest obliczane na podstawie całkowitej i wolnej pojemności wybranego woluminu lokalnego, zgłaszanej przez system operacyjny. Domyślnie jest to wolumin zawierający folder domowy użytkownika. Aby udostępnić wybór dysku, Aplikacja odczytuje nazwy, punkty montowania i pojemność obsługiwanych, zamontowanych woluminów lokalnych, w tym dysków zewnętrznych. Wybrany punkt montowania jest zapisywany lokalnie jako preferencja. Aplikacja nie odczytuje nazw plików ani ich zawartości.

## Przechowywanie danych

Aplikacja przechowuje Twoje preferencje (na przykład to, które wskaźniki mają być wyświetlane, oraz ustawienia wyglądu) lokalnie na Twoim urządzeniu, korzystając ze standardowego mechanizmu konfiguracji systemu operacyjnego.

W systemie macOS Aplikacja zapisuje również wpisy diagnostyczne ze znacznikami czasu w lokalnych plikach `autostart.log` w katalogach dzienników Library, w tym w kontenerze aplikacji. Pomagają one diagnozować problemy z uruchamianiem, uruchamianiem po zalogowaniu i działaniem paska menu. Dzienniki mogą zawierać wersję Aplikacji i identyfikator commitu kompilacji, stan rejestracji elementu logowania, wybrane ustawienia, zdarzenia uruchamiania i paska menu, komunikaty o błędach oraz lokalne ścieżki plików (na przykład pliku wykonywalnego, potwierdzenia zakupu w App Store i blokady procesu). Preferencje i dzienniki diagnostyczne pozostają na Twoim urządzeniu; Aplikacja ich nie przesyła.

## Usługi stron trzecich

Aplikacja:
- nie korzysta z usług analitycznych,
- nie zawiera reklam,
- nie łączy się z zewnętrznymi API,
- nie udostępnia danych stronom trzecim.

## Dostęp do sieci

Aplikacja monitoruje aktywność sieciową, aby wyświetlać prędkość wysyłania i pobierania. Aplikacja:
- nie wykonuje żadnych wychodzących żądań sieciowych,
- nie wysyła żadnych danych przez internet,
- nie ma dostępu do treści Twojego ruchu sieciowego.

## Otwarte oprogramowanie

Better Resource Monitor to oprogramowanie typu open source. Pełny kod źródłowy możesz przejrzeć na [GitHubie](https://github.com/alexx855/better-resource-monitor), aby zweryfikować opisane tu praktyki dotyczące prywatności.

## Zmiany w tej polityce

Jeśli wprowadzimy zmiany w niniejszej polityce prywatności, zaktualizujemy powyższą datę „Ostatnia aktualizacja”.

## Kontakt

Jeśli masz pytania dotyczące tej polityki prywatności, otwórz zgłoszenie w naszym [repozytorium na GitHubie](https://github.com/alexx855/better-resource-monitor/issues).
